#!/usr/bin/env python3

import matplotlib.pyplot as plt
import matplotlib.ticker as mtick
import matplotlib.gridspec as gridspec
import numpy as np

class Test():
	epoch = None
	fp = None
	exact = None
	near_exact = None
	aae = None
	are = None
	bw = None
	cms_with_min_aae = None
	cms_with_min_are = None
	
	#For some .dat formats
	num_flows = None
	solved = None
	
	#For other .dat formats
	epoch_period = None
	nf = None
	lazyFP = None
	lazynf = None
	oldnewFP = None
	oldnewnf = None

	def __init__(self):
		#print("New test initialized")
		self.epoch = []
		self.fp = []
		self.exact = []
		self.near_exact = []
		self.aae = []
		self.are = []
		self.bw = []
		self.cms_with_min_aae = []
		self.cms_with_min_are = []
		
		self.num_flows = []
		self.solved = []
		
		self.epoch_period = []
		self.nf = []
		self.lazyFP = []
		self.lazynf = []
		self.oldnewFP = []
		self.oldnewnf = []

def parseFile_format1(filename):
	test = Test()
	
	f = open(filename, 'r')
	Lines = f.readlines()
	
	count = 0
	# Strips the newline character
	for line in Lines:
		count += 1
		
		#Skip headers
		if count < 2:
			continue
		line = line.strip()
		arr = line.split()[1:]
		
		#Update the test with this row
		test.epoch.append(int(arr[0]))
		test.fp.append(float(arr[1]))
		test.exact.append(float(arr[2]))
		test.near_exact.append(float(arr[3]))
		test.aae.append(float(arr[4]))
		test.are.append(float(arr[5]))
		test.bw.append(float(arr[6]))
		test.cms_with_min_aae.append(float(arr[7]))
		test.cms_with_min_are.append(float(arr[8]))
	
	return test

#64cms format
def parseFile_format2(filename):
	test = Test()
	
	f = open(filename, 'r')
	Lines = f.readlines()
	
	count = 0
	# Strips the newline character
	for line in Lines:
		count += 1
		
		#Skip headers
		if count < 2:
			continue
		line = line.strip()
		arr = line.split()[1:]
		
		#Update the test with this row
		test.epoch.append(int(arr[0]))
		test.num_flows.append(float(arr[1]))
		test.fp.append(float(arr[2]))
		test.solved.append(float(arr[3]))
		test.exact.append(float(arr[4]))
		test.near_exact.append(float(arr[5]))
		test.aae.append(float(arr[6]))
		test.are.append(float(arr[7]))
		test.bw.append(float(arr[8]))
		test.cms_with_min_aae.append(float(arr[9]))
		test.cms_with_min_are.append(float(arr[10]))
	
	return test

#bw.dat format
def parseFile_format3(filename):
	test = Test()
	
	f = open(filename, 'r')
	Lines = f.readlines()
	
	count = 0
	# Strips the newline character
	for line in Lines:
		count += 1
		
		#Skip headers
		if count < 2:
			continue
		line = line.strip()
		arr = line.split()[0:]
		
		#Update the test with this row
		test.epoch_period.append(float(arr[0]))
		test.fp.append(float(arr[1]))
		test.nf.append(float(arr[2]))
		test.lazyFP.append(float(arr[3]))
		test.lazynf.append(float(arr[4]))
		test.oldnewFP.append(float(arr[5]))
		test.oldnewnf.append(float(arr[6]))
		
	return test

#comparison.dat format
def parseFile_format4(filename):
	test = Test()
	
	f = open(filename, 'r')
	Lines = f.readlines()
	
	count = 0
	# Strips the newline character
	for line in Lines:
		count += 1
		
		#Skip headers
		if count < 2:
			continue
		line = line.strip()
		arr = line.split()[0:]
		
		#Update the test with this row
		test.exact.append(float(arr[1]))
		test.aae.append(float(arr[2]))
		test.are.append(float(arr[3]))
		test.bw.append(float(arr[4]))
		
	return test


def newFig():
	plt.style.use('./script/my_style.mplstyle')
	fig, ax = plt.subplots()
	
	return fig,ax

def drawFig(fig, ax, style=0, outfile = "UNNAMED.pdf"):
	plt.legend()
	
	if style == 0: #Small square-ish ones (4 cols)
		fig.set_size_inches(3.0, 2.0)
		plt.gcf().subplots_adjust(bottom=0.22, left=0.25, right=0.94,top=0.975)
		ax.set_xlabel("Epoch")
	elif style == 1: #Larger wider ones (2 cols)
		ax.set_xlabel("Epoch Period [Sec]")
		ax.set_xscale('log')
		ax.set_xticks([0.001, 0.01, 0.1, 1])
		ax.set_xticklabels(["0.001", "0.01", "0.1", "1"])
		fig.set_size_inches(4.2, 1.7)
		plt.gcf().subplots_adjust(bottom=0.28, left=0.18, right=0.99,top=0.97)
	elif style == 2: #Larger wider ones (2 cols) 2-line ylabel
		ax.set_xlabel("Epoch Period [Sec]")
		ax.set_xscale('log')
		ax.set_xticks([0.001, 0.01, 0.1, 1])
		ax.set_xticklabels(["0.001", "0.01", "0.1", "1"])
		fig.set_size_inches(4.2, 1.7)
		plt.gcf().subplots_adjust(bottom=0.28, left=0.22, right=0.99,top=0.97)
	
	plt.savefig(outfile)
	#plt.xlim(0)


def plotLine(ax, x_vals, y_vals, label, line=None, marker=None):
	ax.plot(x_vals, y_vals, label=label, linestyle=line, marker=marker, markersize=5)
	

def fig5():
	test1 = parseFile_format2("data/caida1b-64cms.dat")
	test2 = parseFile_format2("data/caida1b-lazy-64cms.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch, test1.exact, label="Standard BF", line="-.")
	plotLine(ax, test2.epoch, test2.exact, label="Lazy BF", line=":")
	
	ax.yaxis.set_major_formatter(mtick.PercentFormatter(decimals=0))
	ax.set_ylabel("Exact Result Fraction       ")
	
	drawFig(fig, ax, outfile="./figs/fig5.pdf")
	

def fig6():
	test1 = parseFile_format2("data/caida1b-64cms.dat")
	test2 = parseFile_format2("data/caida1b-lazy-64cms.dat")
	test3 = parseFile_format2("data/caida1b-64cms.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch, test1.aae, label="Standard BF", line="-.")
	plotLine(ax, test2.epoch, test2.aae, label="Lazy BF", line=":")
	plotLine(ax, test3.epoch, test3.cms_with_min_aae, label="Traditional CMS", line="--")
	
	ax.set_ylabel("Average Absolute Error       ")
	
	drawFig(fig, ax, outfile="./figs/fig6.pdf")

def fig7():
	test1 = parseFile_format2("data/caida1b-64cms.dat")
	test2 = parseFile_format2("data/caida1b-lazy-64cms.dat")
	test3 = parseFile_format2("data/caida1b-64cms.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch, test1.are, label="Standard BF", line="-.")
	plotLine(ax, test2.epoch, test2.are, label="Lazy BF", line=":")
	plotLine(ax, test3.epoch, test3.cms_with_min_are, label="Traditional CMS", line="--")
	
	ax.set_ylabel("Average Relative Error       ")
	
	drawFig(fig, ax, outfile="./figs/fig7.pdf")

def fig8():
	test1 = parseFile_format2("data/caida1b-64cms.dat")
	test2 = parseFile_format2("data/caida1b-lazy-64cms.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch, test1.fp, label="Standard BF", line="-.")
	plotLine(ax, test2.epoch, test2.fp, label="Lazy BF", line=":")
	
	ax.yaxis.set_major_formatter(mtick.PercentFormatter(decimals=1))
	ax.set_ylabel("False Positive Fraction       ")
	
	drawFig(fig, ax, outfile="./figs/fig8.pdf")

def fig9():
	test1 = parseFile_format2("data/caida1b-lazy-64cms.dat")
	test2 = parseFile_format2("data/caida1b-lazy-32cms.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch, test1.exact, label="Lazy BF 64 CMS", line="-.")
	plotLine(ax, test2.epoch, test2.exact, label="Lazy BF 32 CMS", line=":")
	
	ax.yaxis.set_major_formatter(mtick.PercentFormatter(decimals=0))
	ax.set_ylabel("Exact Result Fraction       ")
	
	drawFig(fig, ax, outfile="./figs/fig9.pdf")

def fig10():
	test1 = parseFile_format2("data/caida1b-lazy-64cms.dat")
	test2 = parseFile_format2("data/caida1b-lazy-32cms.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch, test1.aae, label="Lazy BF 64 CMS", line="-.")
	plotLine(ax, test2.epoch, test2.aae, label="Lazy BF 32 CMS", line=":")
	
	ax.set_ylabel("Average Absolute Error       ")
	
	drawFig(fig, ax, outfile="./figs/fig10.pdf")

def fig11():
	test1 = parseFile_format2("data/caida1b-lazy-64cms.dat")
	test2 = parseFile_format2("data/caida1b-lazy-32cms.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch, test1.are, label="Lazy BF 64 CMS", line="-.")
	plotLine(ax, test2.epoch, test2.are, label="Lazy BF 32 CMS", line=":")
	
	ax.set_ylabel("Average Relative Error       ")
	
	drawFig(fig, ax, outfile="./figs/fig11.pdf")

def fig12():
	test1 = parseFile_format2("data/caida1b-64cms.dat")
	test2 = parseFile_format2("data/caida1b-lazy-64cms.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch, test1.bw, label="Standard BF", line="-.")
	plotLine(ax, test2.epoch, test2.bw, label="Lazy BF", line=":")
	
	ax.set_ylabel("# FlowIDs Reported       ")
	
	ax.set_yticks([60000, 80000, 100000, 120000, 140000])
	ax.set_yticklabels(["60K", "80K", "100K", "120K", "140K"])
	
	drawFig(fig, ax, outfile="./figs/fig12.pdf")
	

def fig13():
	test1 = parseFile_format3("data/BW.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch_period, test1.fp, label="Standard BF", line="-.")
	plotLine(ax, test1.epoch_period, test1.lazyFP, label="Lazy BF", line=":")
	plotLine(ax, test1.epoch_period, test1.oldnewFP, label="Old-New BF", line="--")
	
	ax.yaxis.set_major_formatter(mtick.PercentFormatter(decimals=1))
	
	ax.set_ylabel("False Positive Rate      ")
	
	drawFig(fig, ax, style=1, outfile="./figs/fig13.pdf")
	
	#fig.show()
	

def fig14():
	test1 = parseFile_format3("data/BW.dat")
	
	fig, ax = newFig()
	
	plotLine(ax, test1.epoch_period, test1.nf, label="Standard BF", line="-.")
	plotLine(ax, test1.epoch_period, test1.lazynf, label="Lazy BF", line=":")
	plotLine(ax, test1.epoch_period, test1.oldnewnf, label="Old-New BF", line="--")
	
	ax.set_ylabel("Message Rate\n[Keys/Sec]")
	
	ax.set_yticks([50000, 100000, 150000, 200000, 250000, 300000, 350000])
	ax.set_yticklabels(["50K", "100K", "150K", "200K", "250K", "300K", "350K"])
	
	drawFig(fig, ax, style=2, outfile="./figs/fig14.pdf")
	
	#fig.show()
	#input()

def plotBar(ax, xvals, yvals, ylabel="None"):
	hatches = ["xx", "//", "\\\\", "++"]
	hatch_color = "darkgray"
	#hatches = hatches[0,len(xvals)]
	
	assert len(hatches) >= len(xvals), "Not enough hatches"
	
	for xval,yval,hatch in zip(xvals,yvals,hatches):
		if yval is None:
			#ax.bar(" ", 0) #Omit this datapoint
			ax.bar(xval, 0)
			ax.text(xval, 1000, "N/A", horizontalalignment="center")
		else:
			ax.bar(xval, yval, hatch=hatch, alpha=.99, edgecolor=hatch_color)
	
	if ylabel=="# Flow IDs Reported     ":
		ax.set_yticks([25000, 50000, 75000, 100000, 125000])
		ax.set_yticklabels(["25K", "50K", "75K", "100K", "125K"])
		ax.set_ylabel(ylabel, labelpad=-1)
	
	if ylabel=="AAE":
		ax.set_yscale("log")
		ax.set_yticks([0.01, 0.1, 1, 10])
		ax.set_yticklabels(["0.01", "0.1", "1", "10"])
		ax.set_ylabel(ylabel, labelpad=-7)
	
	if ylabel=="ARE":
		ax.set_yscale("log")
		ax.set_yticks([0.001, 0.01, 0.1, 1, 10])
		ax.set_yticklabels(["0.001", "0.01", "0.1", "1", "10"])
		ax.set_ylabel(ylabel, labelpad=-7)
	
	if ylabel=="Flows w/ No Error    ":
		ax.yaxis.set_major_formatter(mtick.PercentFormatter())
		ax.set_ylabel(ylabel, labelpad=-7)
	
	ax.tick_params(axis='x', labelrotation=60)

def fig15():

	test1 = parseFile_format4("data/comparison.dat")
	xvals = ["FL", "NZE",  "PR", "ES"]
	test1.bw[3] = None
	plt.style.use('./script/my_style.mplstyle')
	fig, axs = plt.subplots(1, 4)
    
	plotBar(axs[0], xvals, test1.exact, "Flows w/ No Error    ")
	plotBar(axs[1], xvals, test1.aae, "AAE")
	plotBar(axs[2], xvals, test1.are, "ARE")
	plotBar(axs[3], xvals, test1.bw, "# Flow IDs Reported     ")

	fig.set_size_inches(6.0, 1.9)
	plt.gcf().subplots_adjust(left=0.098, right=0.99,top=0.97,bottom=0.23,wspace=0.75)
	plt.savefig('./figs/fig15.pdf')

fig5()
fig6()
fig7()
fig8()
fig9()
fig10()
fig11()
fig12()
fig13()
fig14()
fig15()

