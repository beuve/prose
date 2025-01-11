import numpy as np
from numpy.linalg import inv

Q = np.array([[0, 1, 0],[0, 0.29, 0.07], [0, 1, 0]])
R = np.array([[0, 0],[0.55, 0.09], [0, 0]])
I = np.array([[1, 0, 0],[0, 1, 0], [0, 0, 1]])
p = 17.989400

print(p * inv(I - Q))
print(p * inv(I - Q) @ R)
print(inv(I - Q)[0, 1] * 8)
