#!/usr/bin/env perl
use strict;
use warnings;
use File::Temp qw(tempfile);
use File::Path qw(make_path);
use File::Find qw(find);

# CI-owned bounded diagnostics; raw output and tool payloads are never uploaded.
my ($phase, @command) = @ARGV;
die "usage: setup-diagnostics.pl PHASE COMMAND...\n" unless defined($phase) && $phase =~ /\A[a-z0-9-]+\z/ && @command;
my ($raw, $path) = tempfile('wire-setup-XXXXXX', TMPDIR => 1, UNLINK => 1);
my $pid = fork();
die "cannot fork setup: $!\n" unless defined $pid;
if (!$pid) {
    open STDOUT, '>&', $raw or die $!;
    open STDERR, '>&', $raw or die $!;
    exec {$command[0]} @command;
    die "cannot execute setup: $!\n";
}
waitpid($pid, 0);
my $status = $? & 127 ? 128 + ($? & 127) : $? >> 8;
print "$phase: status $status\n";
exit 0 unless $status;

sub diagnostic {
    my ($file) = @_;
    return "missing or unsafe evidence\n" unless -f $file && !-l $file;
    open my $in, '<', $file or return "unreadable evidence\n";
    my $bytes = -s $in;
    seek($in, $bytes > 16384 ? $bytes - 16384 : 0, 0) or die $!;
    read($in, my $text, 16384);
    $text =~ s/[^\x09\x0a\x0d\x20-\x7e]/?/g;
    # Drop private key bodies, environment assignments and common credential forms.
    $text =~ s/-----BEGIN .*?PRIVATE KEY-----.*?(?:-----END .*?PRIVATE KEY-----|\z)/[private key omitted]/sg;
    $text =~ s/^\s*(?:export\s+)?[A-Z_][A-Z0-9_]*=.*$/[environment assignment omitted]/mg;
    $text =~ s/((?:authorization|token|password|secret|credential|api[_-]?key)["'\s]*[:=]\s*)[^\s,;]+/${1}[REDACTED]/ig;
    $text =~ s/\bBearer\s+\S+/Bearer [REDACTED]/ig;
    $text =~ s{(https?://)[^/\s\@]+\@}{${1}[REDACTED]\@}g;
    $text =~ s{(https?://[^\s?]+)\?\S+}{$1?[query omitted]}g;
    $text =~ s/\b[A-Za-z0-9_+\/=.-]{40,}\b/[long value omitted]/g;
    return "[no diagnostic output]\n" unless length $text;
    return ($bytes > 16384 ? "[truncated to final 16384 bytes]\n" : '') . $text;
}
my $excerpt = diagnostic($path);
print STDERR "$phase failed with status $status; bounded sanitized diagnostic:\n$excerpt\n";
my $root = ($ENV{RUNNER_TEMP} // $ENV{TMPDIR} // '/tmp') . '/wire-diagnostics';
my $collection = eval {
    die "redirected diagnostic directory" if -l $root;
    make_path($root);
    my $outpath = "$root/$phase.txt";
    die "diagnostic destination already exists" if -e $outpath || -l $outpath;
    open my $out, '>', $outpath or die $!;
    print $out "phase=$phase status=$status\n$excerpt\n";
    my ($seen, $logs) = (0, 0);
    if (-d '.tools' && !-l '.tools') {
        find({ no_chdir => 1, follow => 0, wanted => sub {
            if (++$seen > 512 || $logs >= 8 || -l $_) { $File::Find::prune = 1; return; }
            return unless -f $_ && /(?:\.log|\/stderr|\/stdout|\/status)\z/;
            ++$logs;
            my $attempt = diagnostic($_);
            print $out "\n--- $_ ---\n", $attempt;
            print STDERR "\nAttempt diagnostic $_:\n$attempt\n";
        }}, '.tools');
    }
    print $out "\nSelection bounded to 512 entries and 8 attempt logs; symlinks, payloads and other files omitted.\n";
    close $out or die $!;
    print STDERR "Sanitized evidence: $outpath (at most 9 excerpts of 16 KiB)\n";
    1;
};
print STDERR "Diagnostic collection failed; no artifact claimed: $@\n" unless $collection;
exit $status;
