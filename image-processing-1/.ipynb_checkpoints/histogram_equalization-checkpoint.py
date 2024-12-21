import math

M = [[3, 5, 6, 2, 3], [1, 6, 4, 1, 3], [7, 7, 2, 2, 1]]

intensities = [0, 1, 2, 3, 4, 5, 6, 7]
intensity_count = [0, 3, 3, 3, 1, 1, 2, 2]

total_pixels = sum(intensity_count)  # 15
print([round(count / total_pixels, 3) for count in intensity_count])

pdf = [0.0, 0.2, 0.2, 0.2, 0.067, 0.067, 0.133, 0.133]  # Probability Density Function

print(
    [round(sum(pdf[: i + 1]), 3) for i in range(len(pdf))]
)  # Sum up the PDF to get the CDF

cdf = [0.0, 0.2, 0.4, 0.6, 0.667, 0.734, 0.867, 1.0]  # Cumulative Distribution Function

print([math.floor(cdf_value * 7) for cdf_value in cdf])

new_intensities = [
    0,
    1,
    2,
    4,
    4,
    5,
    6,
    7,
]  # Round down any resulting pixel intensities that are not integers (use the floor operator)

M_equalized = [[4, 5, 6, 2, 4], [1, 6, 4, 1, 4], [7, 7, 2, 2, 1]]
