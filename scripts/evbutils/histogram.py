import numpy as np
from numpy.typing import NDArray
from dataclasses import dataclass
from typing import Optional

#Utility functions
def clamp_low(x: float, edge: float) -> float:
    return x if x > edge else edge

def clamp_hi(x: float, edge: float) -> float:
    return x if x < edge else edge

def clamp_range(xrange: tuple[float, float], min_max: tuple[float, float]):
    return (clamp_low(xrange[0], min_max[0]), clamp_hi(xrange[1], min_max[1]))

"""
Hist1D, Hist2D
Dataclasses storing histogram data (name, counts per bin, bin edges)
When going to plot use the following idioms:

Hist1D:
    matplotlib.pyplot.stairs(hist.counts, hist.bins)

Hist2D:
    matplotlib.pyplot.pcolormesh(hist.x_bins, hist.y_bins, hist.counts)
"""
@dataclass
class Hist1D:
    name: str
    counts: NDArray[np.float64]
    bins: NDArray[np.float64]

    def get_bin(self, x: float) -> Optional[float]:
        if x < self.bins.min() and x > self.bins.max():
            return None
        
        for i in range(len(self.counts)):
            if x >= self.bins[i] and x < self.bins[i+1]:
                return i
        return None

    #returns (integral, mean, std_dev)
    def stats_for_range(self, xrange: tuple[float, float]) -> Optional[tuple[float, float, float]]:
        clamped_range = clamp_range(xrange, (self.bins.min(), self.bins.max()))
        bin_min = self.get_bin(clamped_range[0])
        bin_max = self.get_bin(clamped_range[1])
        if bin_min is None or bin_max is None:
            return None

        integral = np.sum(self.counts[bin_min:(bin_max+1)])
        mean = np.average(self.bins[bin_min:(bin_max + 1)], weights=self.counts[bin_min:{bin_max+1}])
        variance = np.average(self.bins[bin_min:(bin_max+1)] - mean, weights=self.counts[bin_min:(bin_max+1)])
        return (integral, mean, np.sqrt(variance))

@dataclass
class Hist2D:
    name: str
    counts: NDArray[np.float64]
    x_bins: NDArray[np.float64]
    y_bins: NDArray[np.float64]

"""
Histogrammer
Histogrammer is a wrapper around a dictionary of str->Hist1D|Hist2D
A new histogram can be added to the dictionary using the add_hist1d/add_hist2d methods. The name passed to
these methods is used as the key for the dictionary. To add data to the histograms use the fill_hist1d/fill_hist2d methods.
The fill methods accept arrays of data, and this is by intent. It would not be efficient to fill the histograms point by point. Rather, prefer
passing entire data sets (like dataframe columns). Finally, to retrieve a histogram (for plotting, etc), use the get_hist1d/get_hist2d methods.
Prefer the getters over direct access to the underlying dictionary as the getters perfom some error checking.

Should be pickle-able -> We can save histograms in a concise binary way
"""
class Histogrammer:
    def __init__(self):
        self.histograms: dict[str, Hist1D | Hist2D] = {}

    def add_hist1d(self, name: str, bins: int, range: tuple[float, float]):
        if name in self.histograms:
            print(f"Overwriting histogram named {name} in Histogrammer.add_histogram!")

        hist = Hist1D(name, np.empty(0), np.empty(0))
        hist.counts, hist.bins = np.histogram(a=[], bins=bins, range=range)
        self.histograms[name] = hist

    def add_hist2d(self, name: str, bins: tuple[int, int], ranges: tuple[tuple[float, float], tuple[float, float]]):
        if name in self.histograms:
            print(f"Overwriting histogram named {name} in Histogrammer.add_histogram!")

        hist = Hist2D(name, np.empty(0), np.empty(0), np.empty(0))
        hist.counts, hist.x_bins, hist.y_bins = np.histogram2d(x=[], y=[], bins=bins, range=ranges)
        self.histograms[name] = hist

    def fill_hist1d(self, name: str, data: np.ndarray) -> bool:
        if name not in self.histograms:
            return False

        hist = self.histograms[name]
        if type(hist) is not Hist1D:
            return False

        hist.counts = hist.counts + np.histogram(a=data, bins=hist.bins)[0]
        return True

    def fill_hist2d(self, name: str, x_data: np.ndarray, y_data: np.ndarray) -> bool:
        if name not in self.histograms:
            return False
        
        hist = self.histograms[name]
        if type(hist) is not Hist2D:
            return False
        
        hist.counts += np.histogram2d(x=x_data, y=y_data, bins=(hist.x_bins, hist.y_bins))[0]
        return True

    def get_hist1d(self, name: str) -> Optional[Hist1D]:
        if name not in self.histograms:
            return None
        
        hist = self.histograms[name]
        if type(hist) is not Hist1D:
            return None
        else:
            return hist

    def get_hist2d(self, name: str) -> Optional[Hist2D]:
        if name not in self.histograms:
            return None
        
        hist = self.histograms[name]
        if type(hist) is not Hist2D:
            return None
        else:
            return hist
