import polars
from matplotlib import pyplot, widgets
from evbutils import load_cut_json, write_cut_json, CutHandler, Histogrammer
from pathlib import Path
from typing import Optional

DATA_DIRECTORY: str = "/media/data/gwm17/spsevb_test/built"

#Merge a bunch of runs into one dataframe. This can be useful for doing one-shot analysis,
#but need to be mindful of memory limitations (and performance penalties)
def merge_runs_to_dataframe(run_min: int, run_max: int) -> polars.DataFrame:
    data_path = Path(DATA_DIRECTORY)
    path = data_path / f"run_{run_min}.parquet"
    total_df = polars.read_parquet(path)
    for i in range(run_min+1, run_max+1):
        path = data_path / f"run_{i}.parquet"
        if path.exists():
            total_df.vstack(polars.read_parquet(path), in_place=True)
    total_df.rechunk()
    return total_df

def get_dataframe(run_num: int) -> Optional[polars.DataFrame]:
    data_path = Path(DATA_DIRECTORY)
    path = data_path / f"run_{run_num}.parquet"
    if path.exists():
        return polars.read_parquet(path)
    else:
        return None

#Example plotter making an xavg histogram with an ede gate
def plot(run_min: int, run_max: int):
    ede_cut = load_cut_json("ede_cut.json")
    if ede_cut is None:
        print("blerk, cut invalid couldn't plot")
        return

    grammer = Histogrammer()
    grammer.add_hist1d("xavg", 600, (-300.0, 300.0))
    grammer.add_hist2d("ede", (512, 512), ((0.0, 4096.0), (0.0, 4096.0)))

    for run in range(run_min, run_max+1):
        df = get_dataframe(run)
        if df is None:
            continue
        df_ede = df.filter(polars.col("ScintLeftEnergy").arr.concat("AnodeBackEnergy").map(ede_cut.is_cols_inside))
        grammer.fill_hist1d("xavg", df_ede.select("Xavg").to_numpy())
        grammer.fill_hist2d("ede", df_ede.select("ScintLeftEnergy").to_numpy(), df_ede.select("AnodeBackEnergy").to_numpy())

    fig, ax = pyplot.subplots(1,2)
    fig.suptitle("SPS in Python")
    grammer.draw_hist1d("xavg", ax[0])
    grammer.draw_hist2d("ede", ax[1])
    ax[0].set_xlabel("xavg (mm)")
    ax[0].set_ylabel("counts")
    ax[0].set_title("XAvg From Python!")
    ax[1].set_xlabel("Scint Left")
    ax[1].set_ylabel("Anode Back")
    ax[1].set_title("E-dE")
    pyplot.show()

#Example of scripted cut generation. You have to close the plot window to save the cut
def draw_ede_cut():
    df = polars.read_parquet("/media/data/gwm17/spsevb_test/built/run_184.parquet")
    handler = CutHandler()
    fig, ax = pyplot.subplots(1,1)

    sc_df = df.select(["ScintLeftEnergy","AnodeBackEnergy"]).filter((polars.col("ScintLeftEnergy") != -1e6) & (polars.col("AnodeBackEnergy") != -1e6))

    selector = widgets.PolygonSelector(ax, handler.onselect)

    ax.hist2d(sc_df.select("ScintLeftEnergy").to_series(), sc_df.select("AnodeBackEnergy").to_series(), 256, [[0, 4096],[0,4096]])

    pyplot.show()

    handler.cuts["cut_0"].name = "ede_cut"
    write_cut_json(handler.cuts["cut_0"], "ede_cut.json")

if __name__ == "__main__":
    plot(184, 184)
    #draw_ede_cut()