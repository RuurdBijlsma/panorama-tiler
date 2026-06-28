# Python `generate.py` results:

```text
========================================
 BENCHMARK RESULTS 
========================================
Total Runs:     20
Average Time:   10.3796 seconds
Median Time:    10.0971 seconds
Fastest Run:    9.7104 seconds
Slowest Run:    14.8632 seconds
Std Deviation:  1.1210 seconds
========================================
```

# Rust implementation

```text
Additional Statistics:
	Lower bound 	Estimate 	Upper bound
Slope 	1.0248 s 	1.0274 s 	1.0306 s
R² 	0.9989269 	0.9991317 	0.9988334
Mean 	1.0276 s 	1.0311 s 	1.0350 s
Std. Dev. 	5.2254 ms 	8.7379 ms 	11.353 ms
Median 	1.0257 s 	1.0284 s 	1.0339 s
MAD 	3.3906 ms 	7.2459 ms 	12.316 ms
```

### Performance Comparison

| Metric                 | Python (`generate.py`) | Rust Implementation | 
|:-----------------------|:-----------------------|:--------------------|
| **Mean Time**          | 10.380 s               | 1.031 s             | 
| **Median Time**        | 10.097 s               | 1.028 s             | 
| **Standard Deviation** | 1.121 s                | 8.74 ms             | 
