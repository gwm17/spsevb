import polars
from matplotlib import pyplot
from evbutils import load_cut_json

def plot():
    df = polars.read_parquet("run_290.parquet")
    ede_cut = load_cut_json("edeCut.json")
    if ede_cut is None:
        print("blerk, cut invalid couldn't plot")
        return

    xavg = df.select("Xavg").filter(
        ede_cut.is_cols_inside(polars.col("ScintLeftEnergy"), polars.col("CathodeEnergy"))
    ).to_series()

    fig, ax = pyplot.subplots(1,1)
    ax.hist(xavg, bins=600, range=(-300.0, 300.0))
    ax.set_xlabel("xavg (mm)")
    ax.set_ylabel("counts")
    ax.set_title("XAvg From Python!")
    pyplot.show()

plot()