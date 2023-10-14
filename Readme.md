# FlowLiDAR

This is the repository for the simulator used in the paper "Lightweight Acquisition and Ranging of Flows in the Data-Plane", ACM Sigmetrics 2024. 
The repository contains the simulator, the P4 implementation, and the scripts used to create tables and figures presented in the evaluation section.

# Getting Started

The simulator has been developed on Ubuntu 20.04 using rust 1.68.2. Other distributions or versions may need different steps.

# Build simulator

Run the following command to build the FlowLiDAR simulator:

```
$ cargo build -r
```


# To recreate Figures and tables of section 

First copy the CAIDA traces in the traces dir.

After, run the following command:

```
$ ./reproduce.sh
```

This command will create the log files. After, for each figure a .pdf file is created and for each table a .txt file is created. 

The command also builds the sketches used for comparison: ElasicSketch, FlowRadar, PR-Sketch, and NZE.

ElasticSktch and FlowRadar are taken from [ElasticSketch](https://github.com/BlockLiu/ElasticSketchCode).
PR-Sketch is taken from [PR-Sketch](https://github.com/N2-Sys/PR-Sketch).
NZE is taken from [NZE](https://github.com/N2-Sys/NZE-Sketch).

