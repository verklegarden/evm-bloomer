""""
Test of a spiral potting algorithm
"""

import matplotlib.pyplot as plt
import numpy as np
import time

BLOOM = "1111111111110000111111111111110010000000000000001111111111111111111111111000000011111111111111111111111111111111111111111111111111111111111111111111111111111111111110000000000000000000000000000000000000000000000000000000000000000000000000001111110000100101"
# Initialize the grid size
GRID_SIZE = 16

# Create a 16x16 grid with all zeros
grid = np.zeros((GRID_SIZE, GRID_SIZE))

# Set up the plot
fig, ax = plt.subplots()
cax = ax.imshow(grid, cmap='gray', vmin=0, vmax=1)

# Function to update the grid
def update_grid(grid, index):
    i, j = index
    grid[i, j] = 1
    cax.set_data(grid)
    plt.draw()
    plt.pause(0.1)  # Pause for 1 second


# Move outwards in a spiral from the center
counter  = 0
pos = np.array([7,7])
mag = 1
direction = np.array([1,0])
while counter < 256:
    for i in range(2):
        for i in range(mag):
            if BLOOM[counter] == '1':
                update_grid(grid, (pos[0], pos[1]))
            pos += direction
            counter += 1
        tmp = direction[0]
        direction[0] = -1 *direction[1]
        direction[1] = tmp
    mag += 1





plt.show()
