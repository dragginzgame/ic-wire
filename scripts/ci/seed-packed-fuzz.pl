#!/usr/bin/env perl
use strict;
use warnings;

# Independent frame oracle for Cell variants 8..14 in the qualification fuzz_runtime/mod.rs.
# Keep seeds deterministic and restore them before each run; discovered corpus
# entries are never removed. Partial words deliberately seed malformed admission.
my ($directory) = @ARGV;
defined($directory) && -d $directory or die "usage: $0 EXISTING_CORPUS_DIRECTORY\n";
my @widths = (2, 2, 4, 4, 8, 8, 32);
for my $index (0..$#widths) {
    my $tag = 8 + $index;
    my $width = $widths[$index];
    for my $length (0, $width, 2 * $width, 127, 128, 129, 256, 512) {
        my $remaining = $length;
        my $prefix = '';
        do {
            my $byte = $remaining & 127;
            $remaining >>= 7;
            $prefix .= pack('C', $byte | ($remaining ? 128 : 0));
        } while ($remaining);
        my $payload = join('', map { pack('C', ($_ * 73 + 19) & 255) } 0..$length-1);
        for my $direction (0, 1) {
            my $bytes = pack('C*', 73, 87, 17, 34, 51, 68, 85, 102, 119, 136,
                $direction, $direction ? 0 : 1, $tag) . $prefix . $payload;
            open(my $file, '>:raw', "$directory/packed-$tag-$length-$direction") or die "$!\n";
            print {$file} $bytes or die "$!\n";
            close($file) or die "$!\n";
        }
    }
}
