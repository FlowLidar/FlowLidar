function launch {
	( cd build && make clean )
	rm -dfr .temp/*
	rm -dfr ./build
	./build.sh
	rm $title.log
	{ i=0; for x in /home/guest/caida_passive_2016_20160121/equinix-chicago.dirA.20160121-130000-dir/processed_splitted_pcap_00* ; do ./bin/seq $x > .temp/log$i; i=$(( $i+1 )); done; } 
	{  for x in .temp/* ; do egrep "AAE" $x; done; } | awk -F':' 'BEGIN { acc=0}  { acc+=$2 } END { print "AAE: ",acc/(NR) } ' > $title.log
	{  for x in .temp/* ; do egrep "ARE" $x; done; } | awk -F':' 'BEGIN { acc=0}  { acc+=$2 } END { print "ARE: ",acc/(NR) } ' >> $title.log
	{  for x in .temp/* ; do egrep "cms|evicted" $x; done; } | awk -F' ' 'BEGIN { acc=0}  { acc+=$2 } END { print "BANDWIDTH: ",acc/(NR/2)," keys" } ' >> $title.log
	{  for x in .temp/* ; do egrep "Zero" $x; done; } | awk -F':' 'BEGIN { acc=0}  { acc+=$2 } END { print "SATISFIED: ",acc/(NR),"%" } ' >> $title.log
	cat .temp/log0 | egrep "Total"  >> $title.log
	cat .temp/log0 | egrep "CM"  >> $title.log
	cat .temp/log0 | egrep "Hash"  >> $title.log
	cat .temp/log0 | egrep "Bloom"  >> $title.log
}

if [ ! -x .temp ] ; then
	mkdir .temp
fi

key_size=( 13 ) 
if [[ ${NZE_MEM_FACTOR} == "" ]]; then
	NZE_MEM_FACTOR=1.0
fi
mem_factor=${NZE_MEM_FACTOR}
# 128 KB of baseline memory
baseline_mem=128000
total_mem=$(python -c "
c=float(${baseline_mem})*${mem_factor}
print(c)
")
bloom_mem_perc=0.5
cms_mem_perc=0.25
slot_mem_perc=0.25

bloom_size=$(python -c "
import math
c=math.floor(${total_mem}*${bloom_mem_perc}*8.0)
print(c)
")
slot_num=$(python -c "
import math
c=math.floor((${total_mem}*${slot_mem_perc})/21.0)
print(c)
")

cm_depth=4
cm_width=$(python -c "
import math
c=math.floor(((${total_mem}*${cms_mem_perc})/2)/${cm_depth})
print(c)
")

echo $bloom_size
echo $slot_num
echo $cm_width

for k in ${key_size[@]}; do
	export KEY_T_SIZE=${k}
	rm -dfr nze_run_data
	mkdir nze_run_data
	export CM_DEPTH=${cm_depth}
	export CM_WIDTH=${cm_width}
	export BLOOM_SIZE=${bloom_size}
	export SLOT_NUM=${slot_num}
	title=paper_figure_15
	launch
done

