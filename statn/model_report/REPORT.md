# Complete Trading Model Report

## 1. Data Analysis
### Stationary Test
```

Reading market file...

Market price history read (7782 lines)


Indicator version 0


Trend  min=-0.0291  max=0.0182  0.500 quantile=0.0007


Gap analysis for trend with lookback=10
  Size   Count
     1      53
     2      46
     4     100
     8     253
    16     245
    32     105
    64      13
   128       0
   256       0
   512       0
>  512       0


Volatility  min=0.0031  max=0.0862  0.500 quantile=0.0115


Gap analysis for volatility with lookback=10
  Size   Count
     1      61
     2      22
     4      29
     8      23
    16      42
    32      47
    64      28
   128      22
   256       8
   512       4
>  512       0


 Finished...

```
### Entropy Analysis
```
Reading market file...
Market price history read (7782 lines)

Indicator version 0

Trend  min=-0.0291  max=0.0182  median=0.0007  relative entropy=0.498

Volatility  min=0.0031  max=0.0862  median=0.0115  relative entropy=0.493

Expansion  min=0.0505  max=24.2377  median=0.9950  relative entropy=0.158

RawJump  min=-0.1709  max=0.0791  median=0.0037  relative entropy=0.483

CleanedJump  min=-0.0260  max=0.0532  median=0.0038  relative entropy=0.809


Press Enter to exit...

```

## 2. Model Generation (try_cd_ma)
### Output Summary
```
CD_MA - Moving Average Crossover Indicator Selection

Loading market data...
Training cases: 7510
Test cases: 253
MA indicators: 50
Total indicators: 50
Computing training indicators...
Running 10-fold cross-validation...
Optimal lambda: 0.024430
Training final model...
In-sample explained variance: 0.897%
Computing test indicators...
Evaluating on test set...
OOS total return: 0.00786 (0.789%)

============================================================
Running Backtest
============================================================

Running backtest on test data...
Backtest completed:
  Total trades: 214
  Total return: -18.84%
  Win rate: 56.07%
  Max drawdown: 17.66%
  Sharpe ratio: -0.042
Backtest results written to results/backtest_results.txt

Results written to results/CD_MA.LOG

============================================================
Summary
============================================================

Model Performance:
  In-sample explained variance: 0.897%
  OOS total return: 0.00786 (0.789%)

Backtest Performance:
  Total return: -18.84%
  Total trades: 214
  Win rate: 56.07%
  Max drawdown: 17.66%
  Sharpe ratio: -0.042

```
### Best Parameters
n_long=20, n_short=10, alpha=0.5

## 3. Model Verification
### Monte Carlo Permutation Test
```

Reading market file...
Market price history read
    0: Ret = 1.462  Lookback=19 20  NS, NL=2934 4828  TrndComp=0.6538  TrnBias=0.8077
    1: Ret = 1.554  Lookback=19 20  NS, NL=3409 4353  TrndComp=0.3259  TrnBias=1.2280
    2: Ret = 2.315  Lookback=5 7  NS, NL=3590 4172  TrndComp=0.2009  TrnBias=2.1141
    3: Ret = 1.796  Lookback=5 6  NS, NL=3554 4208  TrndComp=0.2258  TrnBias=1.5705
    4: Ret = 1.561  Lookback=11 13  NS, NL=3485 4277  TrndComp=0.2734  TrnBias=1.2880
    5: Ret = 0.773  Lookback=9 10  NS, NL=3519 4243  TrndComp=0.2499  TrnBias=0.5234
    6: Ret = 2.931  Lookback=1 8  NS, NL=3594 4168  TrndComp=0.1981  TrnBias=2.7329
    7: Ret = 1.453  Lookback=1 10  NS, NL=3564 4198  TrndComp=0.2189  TrnBias=1.2339
    8: Ret = 1.869  Lookback=16 19  NS, NL=3402 4360  TrndComp=0.3307  TrnBias=1.5385
    9: Ret = 2.386  Lookback=6 8  NS, NL=3552 4210  TrndComp=0.2271  TrnBias=2.1588
   10: Ret = 1.477  Lookback=14 15  NS, NL=3481 4281  TrndComp=0.2762  TrnBias=1.2008
   11: Ret = 3.258  Lookback=17 19  NS, NL=3401 4361  TrndComp=0.3314  TrnBias=2.9268
   12: Ret = 2.863  Lookback=18 19  NS, NL=3496 4266  TrndComp=0.2658  TrnBias=2.5976
   13: Ret = 0.842  Lookback=1 2  NS, NL=3565 4197  TrndComp=0.2182  TrnBias=0.6243
   14: Ret = 2.448  Lookback=1 13  NS, NL=3483 4279  TrndComp=0.2748  TrnBias=2.1737
   15: Ret = 2.354  Lookback=1 4  NS, NL=3583 4179  TrndComp=0.2057  TrnBias=2.1478
   16: Ret = 2.312  Lookback=15 20  NS, NL=3414 4348  TrndComp=0.3224  TrnBias=1.9894
   17: Ret = 2.439  Lookback=1 3  NS, NL=3546 4216  TrndComp=0.2313  TrnBias=2.2076
   18: Ret = 1.730  Lookback=12 19  NS, NL=3428 4334  TrndComp=0.3127  TrnBias=1.4177
   19: Ret = 1.158  Lookback=4 5  NS, NL=3564 4198  TrndComp=0.2189  TrnBias=0.9393
   20: Ret = 1.350  Lookback=6 16  NS, NL=3606 4156  TrndComp=0.1899  TrnBias=1.1606
   21: Ret = 2.420  Lookback=11 16  NS, NL=3553 4209  TrndComp=0.2265  TrnBias=2.1939
   22: Ret = 1.619  Lookback=12 17  NS, NL=3364 4398  TrndComp=0.3569  TrnBias=1.2623
   23: Ret = 1.624  Lookback=6 14  NS, NL=3427 4335  TrndComp=0.3134  TrnBias=1.3102
   24: Ret = 0.553  Lookback=15 19  NS, NL=3468 4294  TrndComp=0.2851  TrnBias=0.2682
   25: Ret = 2.138  Lookback=4 7  NS, NL=3562 4200  TrndComp=0.2202  TrnBias=1.9173
   26: Ret = 1.890  Lookback=8 20  NS, NL=3490 4272  TrndComp=0.2699  TrnBias=1.6205
   27: Ret = 2.620  Lookback=10 15  NS, NL=3495 4267  TrndComp=0.2665  TrnBias=2.3530
   28: Ret = 2.574  Lookback=15 16  NS, NL=3490 4272  TrndComp=0.2699  TrnBias=2.3044
   29: Ret = 2.538  Lookback=12 17  NS, NL=3420 4342  TrndComp=0.3183  TrnBias=2.2193
   30: Ret = 2.309  Lookback=18 19  NS, NL=3419 4343  TrndComp=0.3190  TrnBias=1.9902
   31: Ret = 1.365  Lookback=11 20  NS, NL=3406 4356  TrndComp=0.3279  TrnBias=1.0371
   32: Ret = 2.293  Lookback=1 3  NS, NL=3555 4207  TrndComp=0.2251  TrnBias=2.0677
   33: Ret = 2.151  Lookback=2 3  NS, NL=3609 4153  TrndComp=0.1878  TrnBias=1.9630
   34: Ret = 1.939  Lookback=4 5  NS, NL=3484 4278  TrndComp=0.2741  TrnBias=1.6647
   35: Ret = 3.160  Lookback=16 19  NS, NL=3627 4135  TrndComp=0.1754  TrnBias=2.9842
   36: Ret = 1.277  Lookback=9 20  NS, NL=3422 4340  TrndComp=0.3169  TrnBias=0.9601
   37: Ret = 1.444  Lookback=7 12  NS, NL=3493 4269  TrndComp=0.2679  TrnBias=1.1759
   38: Ret = 1.790  Lookback=5 6  NS, NL=3548 4214  TrndComp=0.2299  TrnBias=1.5603
   39: Ret = 1.189  Lookback=13 15  NS, NL=3386 4376  TrndComp=0.3417  TrnBias=0.8473
   40: Ret = 3.495  Lookback=3 4  NS, NL=3528 4234  TrndComp=0.2437  TrnBias=3.2515
   41: Ret = 2.796  Lookback=1 3  NS, NL=3544 4218  TrndComp=0.2327  TrnBias=2.5631
   42: Ret = 2.399  Lookback=2 19  NS, NL=3502 4260  TrndComp=0.2617  TrnBias=2.1370
   43: Ret = 2.716  Lookback=9 18  NS, NL=3446 4316  TrndComp=0.3003  TrnBias=2.4160
   44: Ret = 2.476  Lookback=8 9  NS, NL=3548 4214  TrndComp=0.2299  TrnBias=2.2464
   45: Ret = 2.505  Lookback=1 19  NS, NL=3401 4361  TrndComp=0.3314  TrnBias=2.1734
   46: Ret = 2.254  Lookback=6 7  NS, NL=3495 4267  TrndComp=0.2665  TrnBias=1.9872
   47: Ret = 1.899  Lookback=1 6  NS, NL=3521 4241  TrndComp=0.2485  TrnBias=1.6500
   48: Ret = 2.538  Lookback=14 16  NS, NL=3490 4272  TrndComp=0.2699  TrnBias=2.2678
   49: Ret = 3.350  Lookback=15 18  NS, NL=3447 4315  TrndComp=0.2996  TrnBias=3.0503
   50: Ret = 3.682  Lookback=1 2  NS, NL=3567 4195  TrndComp=0.2168  TrnBias=3.4652
   51: Ret = 2.247  Lookback=3 7  NS, NL=3518 4244  TrndComp=0.2506  TrnBias=1.9959
   52: Ret = 0.227  Lookback=15 16  NS, NL=3525 4237  TrndComp=0.2458  TrnBias=-0.0190
   53: Ret = 1.657  Lookback=7 11  NS, NL=3415 4347  TrndComp=0.3217  TrnBias=1.3357
   54: Ret = 2.406  Lookback=6 8  NS, NL=3547 4215  TrndComp=0.2306  TrnBias=2.1757
   55: Ret = 2.582  Lookback=7 9  NS, NL=3523 4239  TrndComp=0.2472  TrnBias=2.3346
   56: Ret = 2.354  Lookback=3 6  NS, NL=3483 4279  TrndComp=0.2748  TrnBias=2.0790
   57: Ret = 1.440  Lookback=18 19  NS, NL=3447 4315  TrndComp=0.2996  TrnBias=1.1403
   58: Ret = 1.925  Lookback=8 11  NS, NL=3471 4291  TrndComp=0.2831  TrnBias=1.6421
   59: Ret = 1.448  Lookback=4 13  NS, NL=3419 4343  TrndComp=0.3190  TrnBias=1.1290
   60: Ret = 2.002  Lookback=3 6  NS, NL=3586 4176  TrndComp=0.2037  TrnBias=1.7983
   61: Ret = 1.850  Lookback=8 9  NS, NL=3456 4306  TrndComp=0.2934  TrnBias=1.5565
   62: Ret = 3.265  Lookback=1 4  NS, NL=3549 4213  TrndComp=0.2292  TrnBias=3.0360
   63: Ret = 2.328  Lookback=4 15  NS, NL=3551 4211  TrndComp=0.2278  TrnBias=2.0999
   64: Ret = 1.498  Lookback=2 3  NS, NL=3588 4174  TrndComp=0.2023  TrnBias=1.2955
   65: Ret = 1.193  Lookback=13 18  NS, NL=3460 4302  TrndComp=0.2907  TrnBias=0.9025
   66: Ret = 0.914  Lookback=5 6  NS, NL=3543 4219  TrndComp=0.2334  TrnBias=0.6807
   67: Ret = 3.060  Lookback=14 19  NS, NL=3474 4288  TrndComp=0.2810  TrnBias=2.7788
   68: Ret = 1.793  Lookback=4 5  NS, NL=3569 4193  TrndComp=0.2154  TrnBias=1.5776
   69: Ret = 3.171  Lookback=3 8  NS, NL=3557 4205  TrndComp=0.2237  TrnBias=2.9468
   70: Ret = 3.186  Lookback=5 6  NS, NL=3562 4200  TrndComp=0.2202  TrnBias=2.9658
   71: Ret = 3.136  Lookback=13 19  NS, NL=3472 4290  TrndComp=0.2824  TrnBias=2.8541
   72: Ret = 3.338  Lookback=2 6  NS, NL=3539 4223  TrndComp=0.2361  TrnBias=3.1021
   73: Ret = 1.290  Lookback=1 2  NS, NL=3565 4197  TrndComp=0.2182  TrnBias=1.0723
   74: Ret = 1.755  Lookback=1 3  NS, NL=3549 4213  TrndComp=0.2292  TrnBias=1.5254
   75: Ret = 2.140  Lookback=3 12  NS, NL=3498 4264  TrndComp=0.2644  TrnBias=1.8756
   76: Ret = 0.728  Lookback=11 12  NS, NL=3525 4237  TrndComp=0.2458  TrnBias=0.4822
   77: Ret = 0.949  Lookback=2 3  NS, NL=3569 4193  TrndComp=0.2154  TrnBias=0.7335
   78: Ret = 1.980  Lookback=1 2  NS, NL=3564 4198  TrndComp=0.2189  TrnBias=1.7607
   79: Ret = 1.636  Lookback=11 13  NS, NL=3439 4323  TrndComp=0.3052  TrnBias=1.3309
   80: Ret = 1.885  Lookback=8 9  NS, NL=3509 4253  TrndComp=0.2568  TrnBias=1.6283
   81: Ret = 1.436  Lookback=8 12  NS, NL=3461 4301  TrndComp=0.2900  TrnBias=1.1463
   82: Ret = 1.443  Lookback=1 11  NS, NL=3531 4231  TrndComp=0.2416  TrnBias=1.2012
   83: Ret = 2.273  Lookback=13 18  NS, NL=3414 4348  TrndComp=0.3224  TrnBias=1.9507
   84: Ret = 2.238  Lookback=14 15  NS, NL=3427 4335  TrndComp=0.3134  TrnBias=1.9250
   85: Ret = 2.045  Lookback=1 2  NS, NL=3565 4197  TrndComp=0.2182  TrnBias=1.8268
   86: Ret = 2.765  Lookback=6 7  NS, NL=3572 4190  TrndComp=0.2133  TrnBias=2.5516
   87: Ret = 1.661  Lookback=11 13  NS, NL=3571 4191  TrndComp=0.2140  TrnBias=1.4474
   88: Ret = 1.809  Lookback=6 8  NS, NL=3549 4213  TrndComp=0.2292  TrnBias=1.5795
   89: Ret = 2.768  Lookback=14 15  NS, NL=3454 4308  TrndComp=0.2948  TrnBias=2.4735
   90: Ret = 1.423  Lookback=1 2  NS, NL=3565 4196  TrndComp=0.2178  TrnBias=1.2050
   91: Ret = 2.106  Lookback=7 12  NS, NL=3549 4213  TrndComp=0.2292  TrnBias=1.8769
   92: Ret = 1.231  Lookback=1 5  NS, NL=3529 4233  TrndComp=0.2430  TrnBias=0.9877
   93: Ret = 1.356  Lookback=1 3  NS, NL=3547 4215  TrndComp=0.2306  TrnBias=1.1251
   94: Ret = 1.641  Lookback=9 19  NS, NL=3430 4332  TrndComp=0.3114  TrnBias=1.3292
   95: Ret = 2.089  Lookback=7 8  NS, NL=3457 4305  TrndComp=0.2927  TrnBias=1.7966
   96: Ret = 1.142  Lookback=5 6  NS, NL=3594 4168  TrndComp=0.1981  TrnBias=0.9442
   97: Ret = 3.873  Lookback=18 20  NS, NL=3396 4366  TrndComp=0.3348  TrnBias=3.5383
   98: Ret = 1.665  Lookback=2 3  NS, NL=3616 4146  TrndComp=0.1830  TrnBias=1.4819
   99: Ret = 2.391  Lookback=12 14  NS, NL=3514 4248  TrndComp=0.2534  TrnBias=2.1372

7782 prices were read, 100 MCP replications with max lookback = 20

p-value for null hypothesis that system is worthless = 0.7600
Total trend = 2.6794
Original nshort = 2934
Original nlong = 4828
Original return = 1.4615
Trend component = 0.6538
Training bias = 1.7881
Skill = -0.9804
Unbiased return = -0.3266

```
### Sensitivity Analysis
```
Sensitivity analysis successful. See "model_report/SENS.LOG" for details.
```
### Drawdown Analysis
```


1
Mean return
  Actual    Incorrect
   0.001   0.01700
   0.01    0.01700
   0.05    0.06900
   0.1     0.14900

Drawdown
  Actual    Incorrect  Correct
   0.001   0.02900  0.00000
   0.01    0.03000  0.00100
   0.05    0.05400  0.00200
   0.1     0.12900  0.01000


2
Mean return
  Actual    Incorrect
   0.001   0.05950
   0.01    0.05950
   0.05    0.17450
   0.1     0.24350

Drawdown
  Actual    Incorrect  Correct
   0.001   0.04600  0.00000
   0.01    0.06200  0.00100
   0.05    0.14400  0.00950
   0.1     0.21150  0.02100


3
Mean return
  Actual    Incorrect
   0.001   0.04233
   0.01    0.04233
   0.05    0.14933
   0.1     0.20867

Drawdown
  Actual    Incorrect  Correct
   0.001   0.03267  0.00000
   0.01    0.05333  0.00067
   0.05    0.12100  0.00700
   0.1     0.18967  0.01567


4
Mean return
  Actual    Incorrect
   0.001   0.03400
   0.01    0.03400
   0.05    0.12450
   0.1     0.19075

Drawdown
  Actual    Incorrect  Correct
   0.001   0.02600  0.00000
   0.01    0.04350  0.00075
   0.05    0.11425  0.00800
   0.1     0.16900  0.01875


5
Mean return
  Actual    Incorrect
   0.001   0.03080
   0.01    0.03080
   0.05    0.11220
   0.1     0.17480

Drawdown
  Actual    Incorrect  Correct
   0.001   0.02160  0.00020
   0.01    0.03700  0.00100
   0.05    0.10620  0.00800
   0.1     0.16300  0.02220


6
Mean return
  Actual    Incorrect
   0.001   0.02567
   0.01    0.02567
   0.05    0.09633
   0.1     0.15300

Drawdown
  Actual    Incorrect  Correct
   0.001   0.01817  0.00017
   0.01    0.03100  0.00100
   0.05    0.09083  0.00967
   0.1     0.15017  0.02533


7
Mean return
  Actual    Incorrect
   0.001   0.02957
   0.01    0.02957
   0.05    0.09829
   0.1     0.16486

Drawdown
  Actual    Incorrect  Correct
   0.001   0.01986  0.00014
   0.01    0.03986  0.00129
   0.05    0.09886  0.01257
   0.1     0.16300  0.03071


8
Mean return
  Actual    Incorrect
   0.001   0.02725
   0.01    0.02725
   0.05    0.09087
   0.1     0.15163

Drawdown
  Actual    Incorrect  Correct
   0.001   0.01787  0.00013
   0.01    0.03725  0.00137
   0.05    0.09375  0.01263
   0.1     0.15300  0.03087


9
Mean return
  Actual    Incorrect
   0.001   0.02478
   0.01    0.02478
   0.05    0.09222
   0.1     0.14856

Drawdown
  Actual    Incorrect  Correct
   0.001   0.01689  0.00011
   0.01    0.03522  0.00133
   0.05    0.09322  0.01500
   0.1     0.15156  0.03456


10
Mean return
  Actual    Incorrect
   0.001   0.02280
   0.01    0.02280
   0.05    0.09070
   0.1     0.14530

Drawdown
  Actual    Incorrect  Correct
   0.001   0.01580  0.00010
   0.01    0.03700  0.00140
   0.05    0.09460  0.01430
   0.1     0.14930  0.03540

Results written to DRAWDOWN.LOG

```
### Cross Validation
```

Reading market file...

Market price history read


nprices=7782  n_blocks=5  max_lookback=20  n_systems=190  n_returns=7762


nprices=7782  n_blocks=5  max_lookback=20  n_systems=190  n_returns=7762

1000 * Grand criterion = 0.3452  Prob = 0.1667

```
### Confidence Test (Conftest)
```

nsamps=1000  lower_fail_rate=0.100  lower_bound_low_q=0.0900  p=0.1472  lower_bound_high_q=0.1100  p=0.1438

p_of_q=0.010  low_q=0.0791  high_q=0.1231


1

Lower bound fail above=0.000  Lower bound fail below=1.000
Lower bound below lower limit=1.0000  theory p=0.1472  above upper limit=0.0000  theory p=0.1438
Lower p_of_q below lower limit=0.0000  theory p=0.0100  above upper limit=0.0000  theory p=0.0100


Upper bound fail above=1.000  Upper bound fail below=0.000
Upper bound below lower limit=0.0000  theory p=0.1438  above upper limit=1.0000  theory p=0.1472
Upper p_of_q below lower limit=0.0000  theory p=0.0100  above upper limit=0.0000  theory p=0.0100

```
