import polars
from matplotlib import pyplot, widgets
from evbutils import load_cut_json, write_cut_json, CutHandler
from pathlib import Path

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

#Example plotter making an xavg histogram with an ede gate
def plot(run_min: int, run_max: int):
    df = merge_runs_to_dataframe(run_min, run_max)
    print(df)
    ede_cut = load_cut_json("ede_cut.json")
    if ede_cut is None:
        print("blerk, cut invalid couldn't plot")
        return

    #Notes on filtering with cuts: You have to concat the two columns and then map them with the cut function
    df_ede = df.filter(polars.col("ScintLeftEnergy").arr.concat("AnodeBackEnergy").map(ede_cut.is_cols_inside))

    xavg = df_ede.select("Xavg").to_numpy()
    print(df_ede.select(["ScintLeftEnergy","AnodeBackEnergy"]))

    fig, ax = pyplot.subplots(1,1)
    ax.hist(xavg, bins=300, range=(-300.0, 300.0))
    ax.set_xlabel("xavg (mm)")
    ax.set_ylabel("counts")
    ax.set_title("XAvg From Python!")
    pyplot.show()

#Example of scripted cut generation. You have to close the plot window to save the cut
def draw_ede_cut():
    df = polars.read_parquet("/media/data/gwm17/spsevb_test/built/run_139.parquet")
    handler = CutHandler()
    fig, ax = pyplot.subplots(1,1)

    sc_df = df.select(["ScintLeftEnergy","AnodeBackEnergy"]).filter((polars.col("ScintLeftEnergy") != -1e6) & (polars.col("AnodeBackEnergy") != -1e6))

    selector = widgets.PolygonSelector(ax, handler.onselect)

    ax.hist2d(sc_df.select("ScintLeftEnergy").to_series(), sc_df.select("AnodeBackEnergy").to_series(), 256, [[0, 4096],[0,4096]])

    pyplot.show()

    handler.cuts["cut_0"].name = "ede_cut"
    write_cut_json(handler.cuts["cut_0"], "ede_cut.json")

plot(139, 163)
#draw_ede_cut()

