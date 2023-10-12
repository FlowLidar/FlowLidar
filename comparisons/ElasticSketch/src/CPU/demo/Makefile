GCC = g++
CFLAGS = -O2 -std=c++14 ../common/BOBHash32.cpp
SSEFLAGS = -msse2 -mssse3 -msse4.1 -msse4.2 -mavx -march=native
FILES = elastic.out flowradar.out pcap_analyzer

all: $(FILES) 

elastic.out: elastic.cpp
	$(GCC) $(CFLAGS) $(SSEFLAGS) -o elastic.out elastic.cpp 

flowradar.out:
	$(GCC) $(CFLAGS) -o flowradar.out flowradar.cpp	

pcap_analyzer:
	$(GCC) pcap_analyzer.cpp   -o pcap_analyzer -lpcap

clean:
	rm $(all) -f *~ *.o *.out ${FILES}
