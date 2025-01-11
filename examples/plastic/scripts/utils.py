import csv
import numpy as np

def integ(f, t, step):
    return np.sum(f[:t]) * step

def read_csv(path):
  data, t = [],[]
  with open(path) as csv_file:
    csv_reader = csv.reader(csv_file, delimiter=',')
    for line, row in enumerate(csv_reader):
        if line == 0:
           continue
        data.append(int(row[1]))
        t.append(int(row[0]))
  return data,t
        
