set datafile separator ","
set key autotitle columnhead
set size ratio -1
plot "eta3.csv" using 1:2 with lines
while (1) {
    replot
    pause 0.1
}
