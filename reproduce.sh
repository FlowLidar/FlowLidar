#!/bin/bash 

rm data/*dat logs/*log figs/*pdf table*
cargo build -r

echo "Reproduces data for figures 5-14 of the FlowLidar paper"
echo "Send simulations in parallel to create log files"
./target/release/FlowLidar -f ./traces/caida1.pcap > logs/caida1b-64cms.log &
./target/release/FlowLidar -f ./traces/caida1.pcap -l > logs/caida1b-lazy-64cms.log &
./target/release/FlowLidar --num_cms 32 -f ./traces/caida1.pcap -l > logs/caida1b-lazy-32cms.log &
echo "Wait end of simulations"
wait

echo "Create dat files for fig 5-12"
for f in logs/*.log ; 
	do 
		file=`basename $f .log`  
        	cat logs/${file}.log | grep stat > ./data/${file}.dat 
done



echo "Create dat files for fig 13-14"
printf "epoch_period\tFP\tnf\tlazyFP\tlazynf\toldnewFP\toldnewnf\n" | tee data/BW.dat
i=2
for e in 0.001 0.002 0.004 0.008 0.016 0.032 0.064 0.128 0.256 0.512 1.000
do
        echo -n "$e" | tee -a data/BW.dat
        ./target/release/FlowLidar --skip --cms_size 128 --num_cms 1 --bf_size $((1024*$i)) -e $e -f ./traces/caida1.pcap | grep stat | grep -v BW | awk 'BEGIN {nf=0; fp=0} {fp += $4; nf +=$10} END{printf "\t\t%.6f\t %.0f\t", fp/NR, nf/NR}' | tee -a data/BW.dat
        ./target/release/FlowLidar -l --skip --cms_size 128 --num_cms 1 --bf_size $((1024*$i)) -e $e -f ./traces/caida1.pcap | grep stat | grep -v BW | awk 'BEGIN {nf=0; fp=0} {fp += $4; nf +=$10} END{printf "%.6f\t %.0f\t", fp/NR, nf/NR}'  | tee -a data/BW.dat
        ./target/release/FlowLidar -o --skip --cms_size 128 --num_cms 1 --bf_size $((1024*$i)) -e $e -f ./traces/caida1.pcap | grep stat | grep -v BW | awk 'BEGIN {nf=0; fp=0} {fp += $4; nf +=$10} END{printf "%.6f\t %.0f\n", fp/NR, nf/NR}'  | tee -a data/BW.dat
        i=$((2*i))
done


echo "Simulations for fig15 (comparison)"
echo "Step 1: build ES and FR executables"
make
echo "Step 2: convert pcap"
./comparisons/ElasticSketch/src/CPU/demo/pcap_analyzer traces/caida1.pcap caida0.dat
echo "Step 3: run ElasticSketch"
./comparisons/ElasticSketch/src/CPU/demo/elastic.out 0 caida > logs/elastic.log
echo "step 4: compile and run NZE"
./comparisons/NZE-Sketch/reproduce.sh > logs/nze.log
echo "Step 5: compile and run PR-Sketch"
./comparisons/PR-Sketch/reproduce.sh > logs/prsketch.log

#grep FlowLidar data from previous simulations
printf "name\texact\taae\tare\tBW\n" | tee data/comparison.dat
cat logs/caida1b-lazy-32cms.log | grep stat | grep -v Epoch | awk '{noerr+=$6; aae+=$8; are+=$9; bw+=$10;} END{print "FlowLidar:\t" noerr/NR, aae/NR, are/NR, bw/NR;}'   | tee -a data/comparison.dat
cat logs/nze.log | awk '{print "NZE:\t", $1, $2, $3, $4}' | tee -a data/comparison.dat
cat logs/prsketch.log | awk '{print "PR-Sketch:\t", $1, $2, $3, $4}' | tee -a data/comparison.dat
cat logs/elastic.log | grep stat | awk '{print "ElasticSketch:\t", $2, $3, $4, $5 }' | tee -a data/comparison.dat

echo "Create figures"
python3 ./script/paper_figures.py


### script for table5 (Table 5:  Analysis of lazy update benefit)
echo "Reproduce data for table 5"
echo "Send simulations in parallel to create log files"
for trace in caida1 caida2 caida3
do

	./target/release/FlowLidar --kbf 4 --bf_size 524288 --cms_size 256 --num_cms 32 -f ./traces/${trace}.pcap -l > logs/${trace}_32cms_4kbf.log &
	./target/release/FlowLidar --kbf 4 --bf_size 524288 --cms_size 128 --num_cms 64 -f ./traces/${trace}.pcap -l > logs/${trace}_64cms_4kbf.log &

	./target/release/FlowLidar --kbf 6 --bf_size 393216 --cms_size 256 --num_cms 32 -f ./traces/${trace}.pcap -l > logs/${trace}_32cms_6kbf.log &
	./target/release/FlowLidar --kbf 6 --bf_size 393216 --cms_size 128 --num_cms 64 -f ./traces/${trace}.pcap -l > logs/${trace}_64cms_6kbf.log &

	./target/release/FlowLidar --kbf 8 --bf_size 262144 --cms_size 256 --num_cms 32 -f ./traces/${trace}.pcap -l > logs/${trace}_32cms_8kbf.log &
	./target/release/FlowLidar --kbf 8 --bf_size 262144 --cms_size 128 --num_cms 64 -f ./traces/${trace}.pcap -l > logs/${trace}_64cms_8kbf.log &
done
echo "Wait end of simulations"
wait

echo "create table 5"
echo  -e "File\tFP\tAAE\tARE\tBW" | tee table5.txt
for f in logs/*kbf*.log ; 
	do file=`basename $f .log`  
	echo -n $file " " | tee -a table5.txt
	grep stat logs/${file}.log | grep -v Epoch | awk '{fp+=$4; aae+=$8; are+=$9; bw+=$10;} END{print fp/NR, aae/NR, are/NR, bw/NR;}' | tee -a table5.txt
done



## script for table6 (Table 6:  Performance of FlowLiDAR approximate CMS resolution )
echo "Reproduce data for table 6"
echo "Send simulations in parallel to create log files"

./target/release/FlowLidar --approx --cms_size 256 --num_cms 32 -f ../../tracce/caida1.pcap -l > logs/caida1b-lazy-approx1.log &
./target/release/FlowLidar --approx --cms_size 128 --num_cms 32 -f ../../tracce/caida1.pcap -l > logs/caida1b-lazy-approx2.log &
./target/release/FlowLidar --approx --cms_size 64 --num_cms 32 -f ../../tracce/caida1.pcap -l > logs/caida1b-lazy-approx3.log &
./target/release/FlowLidar --approx --cms_size 32 --num_cms 32 -f ../../tracce/caida1.pcap -l > logs/caida1b-lazy-approx4.log &
./target/release/FlowLidar --approx --cms_size 16 --num_cms 32 -f ../../tracce/caida1.pcap -l > logs/caida1b-lazy-approx5.log &
echo "Wait end of simulations"
wait

echo "create table 6"
echo  -e "File\texact\tAAE\tAAE std\tARE\tARE std" | tee table6.txt
for f in logs/*approx*.log ; 
	do file=`basename $f .log`  
	echo -n $file " " | tee -a table6.txt
	grep stat logs/${file}.log | grep -v Epoch | awk '{noerr+=$6; aae+=$8; aae_std+=$11; are+=$9; are_std+=$12;} END{print noerr/NR, aae/NR, aae_std/NR, are/NR, are_std/NR;}' | tee -a table6.txt
done

