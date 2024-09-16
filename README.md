# Gaussian_binomial_coefficient
Compute Gaussian binomial coefficient exactly using Rust.

## Algorithm
The current implementation uses recursive relation of $\binom{m+n}{m}_q$:

$$ \binom{m+n}{m}_q = \binom{m+n-1}{m}_q q^m+ \binom{m+n-1}{m-1}_q$$

To compute up to $\binom{2n}{n}_q$, the computation complexity is $O(n^4), due to
* we compute $(n^2+1)$ terms for each $n$,
* and each requires to compute all $\binom{m'+n'}{m'}_q$ that $m’,n’<=n$.

The memory usage complexity is $O(n^3)$, since it caches all $\binom{m+n}{m}_q$ and $\binom{m+n-1}{n-1}_q$, where $m=0,…,n-1$, when computing $\binom{m+n}{m}_q$. This can be reduced by caching more aggresively.

## Performance (single thread)
* It takes about 4.5 min and 3.2 GB RAM to compute $\binom{512}{256}_q$.
* It takes about 20 min and 8 GB RAM to compute $\binom{736}{368}_q$.
* It takes about 50 min and 18 GB RAM to compute $\binom{932}{416}_q$.

The largest integral coefficient of $\binom{932}{416}_q$ is the integral cofficient of $q^{416*416/2}$, which is about $e^{564}$, or $10^{245}$.
