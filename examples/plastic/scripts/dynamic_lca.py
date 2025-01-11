import numpy as np
from mfa import mfa
from cfa import cfa

def co2(t) :
   return 0.0198 * (0.1756 
                  + 0.1375 * np.exp(-t / 421.093) 
                  + 0.1858 * np.exp(-t / 70.5965) 
                  + 0.2423 * np.exp(-t / 21.4216)
                  + 0.2589 * np.exp(-t / 3.4154))

def compute_rf_curve(data, t, co2):
    res = np.zeros(len(t))
    for (i, q) in enumerate(data):
        print(q)
        res[i+1:] += co2[:len(t)-1-i] * q
    return res


if __name__ == "__main__":
  import matplotlib.pyplot as plt
  t_max = 110
  pt_per_year = 10
  token_in_ton = 8
  N = t_max * pt_per_year
  t = np.linspace(0, t_max, N)
  dt = t[1] - t[0]
  path = "logs/log/recycling/plastic/reentrances.csv"
  I_R_MFA = mfa(t)
  I_R_CFA = np.array(cfa(path, N)) / token_in_ton
  Radiative_forcing_MFA = compute_rf_curve(I_R_MFA, t, co2(t)) * 1000
  Radiative_forcing_CFA = compute_rf_curve(I_R_CFA, t, co2(t)) * 1000
  fig, axs = plt.subplots(2)
  axs[0].set_xlabel("Time (year)")
  axs[0].set_ylabel("Reentry")
  axs[0].plot(t, I_R_CFA, label="CFA", color='#72CAFF')
  axs[0].plot(t, I_R_MFA, label="MFA", color='#5C7140', linestyle='--', dashes=(5, 5))
  axs[0].legend()
  axs[1].set_xlabel("Time (year)")
  axs[1].set_ylabel("Radiative forcing")
  axs[1].plot(t, Radiative_forcing_CFA, label="CFA", color='#72CAFF')
  axs[1].plot(t, Radiative_forcing_MFA, label="MFA", color='#5C7140', linestyle='--', dashes=(5, 5))
  axs[1].legend()
  fig.tight_layout()
  plt.show()
