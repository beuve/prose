import numpy as np
from pathlib import Path
from utils import read_csv

def cfa(path, max_t):
  data, _ = read_csv(path)
  return data[:max_t]

#def co2(t) :
#   return 0.0198 * (0.1756 
#                  + 0.1375 * np.exp(-t / 421.093) 
#                  + 0.1858 * np.exp(-t / 70.5965) 
#                  + 0.2423 * np.exp(-t / 21.4216)
#                  + 0.2589 * np.exp(-t / 3.4154))

#def compute_rf_curve(data, t, co2):
#    res = np.zeros(len(t))
#    for (i, q) in enumerate(data):
#        res[i+1:] += co2[:len(t)-1-i] * q
#    return res

# Displaying the results of prose use case simulation
if __name__ == "__main__":
  import matplotlib.pyplot as plt
  
  print("Starting simulation log interpretation")
   
  t_max = 120 # Total observation duration in months
  pt_per_time_unit = 1 # Number of observation points per months
  token_in_device = 1. # Conversion of tokens in target unit (here, 1 device)
  N = t_max * pt_per_time_unit # Total number of observation points
  t = np.linspace(0, t_max, N) # Vector of time points
  dt = t[1] - t[0] # Distance between 2 observation points
  
  use_reentries_path= "examples/electronics/logs/internet_box/use/device/reentrances.csv"
  use_occupancy_path= "examples/electronics/logs/internet_box/use/device/occupency.csv"

  use_reentries = np.array(cfa(use_reentries_path, N)) / token_in_device
  use_occupancy = np.array(cfa(use_occupancy_path, N)) / token_in_device

  fig, axs = plt.subplots(1,2,figsize=(10, 3.5))

  axs[0].ticklabel_format(axis="y", scilimits=[-3, 3])
  axs[0].set_ylim([0, 4e5])
  axs[0].set_xlabel("Time (month)")
  axs[0].set_ylabel("Use Reentries (devices)")
  axs[0].plot(t, use_reentries, label="CPN (r)", color='#72CAFF',)

  axs[1].set_ylim([0, 4e6])
  axs[1].set_xlabel("Time (month)")
  axs[1].set_ylabel("Use Occupancy (devices)")
  axs[1].plot(t, use_occupancy, label="CPN (r)", color='#72CAFF',)

  # Display graph of components reentry and occupancy
  handles, labels = plt.gca().get_legend_handles_labels()
  
  order = [0]
  axs[1].legend([handles[idx] for idx in order],[labels[idx] for idx in order])
  axs[0].legend([handles[idx] for idx in order],[labels[idx] for idx in order])

  fig.tight_layout()

  Path("examples/electronics/outputs").mkdir(parents=True, exist_ok=True)
  plt.savefig("examples/electronics/outputs/use_cfa.pdf")
  
#####
  repair_reentries_path= "examples/electronics/logs/internet_box/repair/device/reentrances.csv"
  repair_occupancy_path= "examples/electronics/logs/internet_box/repair/device/occupency.csv"

  repair_reentries = np.array(cfa(repair_reentries_path, N)) / token_in_device
  repair_occupancy = np.array(cfa(repair_occupancy_path, N)) / token_in_device

  fig, axs = plt.subplots(1,2,figsize=(10, 3.5))

  axs[0].ticklabel_format(axis="y", scilimits=[-3, 3])
  axs[0].set_ylim([0, 4e5])
  axs[0].set_xlabel("Time (month)")
  axs[0].set_ylabel("Repair Reentries (devices)")
  axs[0].plot(t, repair_reentries, label="CPN (r)", color='#72CAFF',)

  axs[1].set_ylim([0, 4e4])
  axs[1].set_xlabel("Time (month)")
  axs[1].set_ylabel("Repair Occupancy (devices)")
  axs[1].plot(t, repair_occupancy, label="CPN (r)", color='#72CAFF',)

  # Display graph of components reentry and occupancy
  handles, labels = plt.gca().get_legend_handles_labels()
  
  order = [0]
  axs[1].legend([handles[idx] for idx in order],[labels[idx] for idx in order])
  axs[0].legend([handles[idx] for idx in order],[labels[idx] for idx in order])

  fig.tight_layout()

  Path("examples/electronics/outputs").mkdir(parents=True, exist_ok=True)
  plt.savefig("examples/electronics/outputs/repair_cfa.pdf")
