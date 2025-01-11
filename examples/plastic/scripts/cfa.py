from utils import read_csv
import numpy as np

def cfa(path, max_t):
  data, _ = read_csv(path)
  return data[:max_t]


if __name__ == "__main__":
  import matplotlib.pyplot as plt
  t_max = 130
  pt_per_year = 10
  N = t_max * pt_per_year
  t = np.linspace(0, t_max, N)
  dt = t[1] - t[0]
  I_R = cfa("logs/plastic/log/recycling/plastic/reentrances.csv", N)
  plt.plot(t, I_R, label="I^R(t)")
  plt.legend()
  plt.xlabel("Time (t)")
  plt.ylabel("Value")
  plt.title("Solution of the System")
  plt.show()
