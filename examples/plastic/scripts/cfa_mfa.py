import numpy as np
from mfa import mfa
from cfa import cfa
from pathlib import Path

def co2(t) :
   return 0.0198 * (0.1756 
                  + 0.1375 * np.exp(-t / 421.093) 
                  + 0.1858 * np.exp(-t / 70.5965) 
                  + 0.2423 * np.exp(-t / 21.4216)
                  + 0.2589 * np.exp(-t / 3.4154))

def compute_rf_curve(data, t, co2):
    res = np.zeros(len(t))
    for (i, q) in enumerate(data):
        res[i+1:] += co2[:len(t)-1-i] * q
    return res


if __name__ == "__main__":
  import matplotlib.pyplot as plt
  t_max = 110
  pt_per_year = 10
  token_in_ton = 8.
  N = t_max * pt_per_year
  t = np.linspace(0, t_max, N)
  dt = t[1] - t[0]
  path_I_R_R= "examples/plastic/logs/random/recycling/plastic/reentrances.csv"
  path_I_R_C= "examples/plastic/logs/constant/recycling/plastic/reentrances.csv"
  path_K_U_R= "examples/plastic/logs/random/use/plastic/occupency.csv"
  path_K_U_C= "examples/plastic/logs/constant/use/plastic/occupency.csv"
  I_R_MFA, K_U_MFA = np.array(mfa(t))
  I_R_CFA_C = np.array(cfa(path_I_R_C, N)) / token_in_ton
  I_R_CFA_R = np.array(cfa(path_I_R_R, N)) / token_in_ton
  K_U_CFA_C = np.array(cfa(path_K_U_C, N)) / token_in_ton
  K_U_CFA_R = np.array(cfa(path_K_U_R, N)) / token_in_ton
  fig, axs = plt.subplots(2)
  axs[0].ticklabel_format(axis="y", scilimits=[-3, 3])
  axs[0].set_ylim([0, 30000])
  axs[0].set_xlabel("Time (year)")
  axs[0].set_ylabel("Reentry (ton)")
  axs[0].plot(t, I_R_CFA_C, label="CPN (c)", color='#E0E0E0', linestyle='-.')
  axs[0].plot(t, I_R_CFA_R, label="CPN (r)", color='#72CAFF',)
  axs[0].plot(t, I_R_MFA, label="MFA (I_R)", color='#5C7140', linestyle='--', dashes=(5, 5))
  axs[0].legend()
  axs[1].set_xlabel("Time (year)")
  axs[1].set_ylabel("Occupency (ton)")
  axs[1].plot(t, K_U_CFA_C, label="CPN (c)", color='#E0E0E0', linestyle='-.')
  axs[1].plot(t, K_U_CFA_R, label="CPN (r)", color='#72CAFF',)
  axs[1].plot(t, K_U_MFA, label="MFA (K_U)", color='#5C7140', linestyle='--', dashes=(5, 5))
  axs[1].legend()
  fig.tight_layout()
  Path("examples/plastic/outputs").mkdir(parents=True, exist_ok=True)
  plt.savefig("examples/plastic/outputs/cfa_mfa.pdf")
