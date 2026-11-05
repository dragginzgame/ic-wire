#!/usr/bin/env perl
# Project the admitted Cargo group names; Shared owns roster validation.
use strict;
use warnings;
use FindBin qw($RealBin);
use Cwd qw(abs_path);
use File::Temp qw(tempdir);
use JSON::PP;

my $root = abs_path("$RealBin/../..") // die "missing Wire root\n";
chdir $root or die "enter Wire root: $!\n";
mkdir 'target' unless -e 'target';
-d 'target' && !-l 'target' or die "invalid roster scratch directory\n";
my $scratch = tempdir('wire-validation-roster.XXXXXX', DIR => 'target', CLEANUP => 1);
-f 'ci/cargo-release-groups.json' && !-l 'ci/cargo-release-groups.json'
    or die "Cargo group policy must be a regular file\n";
open my $in, '<', 'ci/cargo-release-groups.json' or die "read Cargo groups: $!\n";
my $groups = do { local $/; decode_json(<$in>) };
close $in or die "close Cargo groups: $!\n";
open my $out, '>', "$scratch/groups.json" or die "write group projection: $!\n";
print {$out} JSON::PP->new->canonical->encode([map { $_->{name} } @{$groups->{groups}}]), "\n"
    or die "write group projection: $!\n";
close $out or die "close group projection: $!\n";
my $status = system($^X, 'scripts/ci/validation-evidence.pl', 'roster', $root,
    'ci/release-validation.json', "$scratch/groups.json");
$status >= 0 && !($status & 127) or die "shared roster observation failed\n";
exit($status >> 8);
