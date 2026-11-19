#!/usr/bin/env perl
use strict;
use warnings;
use FindBin;
use lib "$FindBin::Bin/../../vendor/toko-tooling/scripts/ci";
use SetupDiagnostics qw(run_setup excerpt);
use File::Path qw(make_path);
use File::Find qw(find);
use Fcntl qw(O_RDONLY O_NOFOLLOW O_NONBLOCK);
use constant MAX_ATTEMPT_BYTES => 1024 * 1024;

# CI-owned bounded diagnostics; raw output and tool payloads are never uploaded.
my ($phase, @command) = @ARGV;
die "usage: setup-diagnostics.pl PHASE COMMAND...\n" unless defined($phase) && $phase =~ /\A[a-z0-9-]+\z/ && @command;
# Consumer-specific environment filtering and artifact size policy.
sub bounded {
    my ($text) = @_;
    $text =~ s/^\s*(?:export\s+)?[A-Z_][A-Z0-9_]*=.*$/[environment assignment omitted]/mg;
    $text =~ s/\b[A-Za-z0-9_+\/=.-]{40,}\b/[long value omitted]/g;
    return length($text) <= 16384 ? $text : "[truncated]\n" . substr($text, -(16384 - 12));
}

sub attempt_excerpt {
    my ($path) = @_;
    sysopen(my $in, $path, O_RDONLY | O_NOFOLLOW | O_NONBLOCK)
        or return "[attempt log unavailable]\n";
    return "[attempt log unavailable]\n" unless -f $in;
    my $size = -s $in;
    return "[attempt log skipped: $size bytes exceeds 1 MiB]\n" if $size > MAX_ATTEMPT_BYTES;
    my $log = excerpt($in, MAX_ATTEMPT_BYTES);
    return "[attempt log changed during collection]\n" unless (-s $in) == $size;
    return bounded($log);
}

sub collect {
    my ($status, $log) = @_;
    my $excerpt = bounded($log);
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
                my $attempt = attempt_excerpt($_);
                print $out "\n--- attempt $logs ---\n", $attempt;
                print STDERR "\nAttempt diagnostic $logs:\n$attempt\n";
            }}, '.tools');
        }
        print $out "\nSelection bounded to 512 entries and 8 attempt logs; symlinks, payloads and other files omitted.\n";
        close $out or die $!;
        print STDERR "Sanitized evidence: $outpath (at most 9 excerpts of 16 KiB)\n";
        1;
    };
    print STDERR "Diagnostic collection failed; no artifact claimed: $@\n" unless $collection;
}
my $status = run_setup(\&collect, @command);
print "$phase: status $status\n";
exit $status;
