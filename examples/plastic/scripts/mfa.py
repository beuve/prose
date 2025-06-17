import numpy as np
from scipy.stats import lognorm
from tqdm import tqdm


def mfa(times, mu=8, sigma=2, total_tons=1798940, max_iterations=100):
  sigma = np.sqrt(np.log(1 + (sigma / mu)**2))
  mu = np.log(mu) - 0.5 * sigma**2

  dt = times[1] - times[0]
  L = [lognorm.pdf(time, sigma, scale=np.exp(mu)) for time in times]

  O_P = np.zeros_like(times)
  time_length = np.sum(times<1)
  O_P[times < 1] = total_tons / time_length

  O_U = np.zeros_like(times)
  for i in tqdm(range(max_iterations)): 
      O_U_new = np.zeros_like(times)
      for t in range(len(times)):
          outputs = 0
          for tau in range(t):
              outputs += (O_P[tau] + 0.36 * O_U[tau]) * L[t - tau] * dt
          O_U_new[t] = outputs
      
      if np.max(np.abs(O_U_new - O_U)) < 1e-6:
          break
      O_U = O_U_new

  # Compute other variables
  I_R = 0.07 * O_U
  O_R = I_R
  I_I = 0.55 * O_U
  I_D = 0.09 * O_U
  I_U = O_P + 0.36 * O_U
  dK = I_U - O_U
  K_U = [np.sum((I_U - O_U)[:t]) for t in range(len(times))]

  return I_R, K_U

if __name__ == "__main__":
  import matplotlib.pyplot as plt
  t_max = 130
  pt_per_year = 10
  N = t_max * pt_per_year
  times = np.linspace(0, t_max, N)
  dt = times[1] - times[0]
  I_R, _ = mfa(times)
  plt.plot(times, I_R, label=f"I^R(t)")
  plt.legend()
  plt.xlabel("Time (t)")
  plt.ylabel("Value")
  plt.title("Solution of the System")
  plt.show()
