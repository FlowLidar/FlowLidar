#!/usr/bin/bash

dir=./comparisons/NZE-Sketch

# build NZE with default memory values
$dir/build.sh 1>/dev/null 2>/dev/null
# prepare .temp folder
if [ ! -x $dir/.temp ] ; then
	mkdir $dir/.temp
fi
rm $dir/.temp/*
# launch simulation for all seconds in the trace
{ i=0; for x in $dir/trace_1/processed_splitted_pcap_00* ; do $dir/bin/seq $x > $dir/.temp/log$i; i=$(( $i+1 )); done; } 
aae=$({  for x in $dir/.temp/* ; do egrep "AAE" $x; done; } | awk -F':' 'BEGIN { acc=0}  { acc+=$2 } END { print acc/(NR) } ')
are=$({  for x in $dir/.temp/* ; do egrep "ARE" $x; done; } | awk -F':' 'BEGIN { acc=0}  { acc+=$2 } END { print acc/(NR) } ')
bw=$({  for x in $dir/.temp/* ; do egrep "cms|evicted" $x; done; } | awk -F' ' 'BEGIN { acc=0}  { acc+=$2 } END { print acc/(NR/2) } ')
noerr=$({  for x in $dir/.temp/* ; do egrep "Zero" $x; done; } | awk -F':' 'BEGIN { acc=0}  { acc+=$2 } END { print acc/(NR) } ')
echo $noerr $aae $are $bw

