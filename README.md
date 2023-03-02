# spsevb

spsevb is a Rust based event builder for data from the Super-Enge Split-Pole Spectrograph at FSU. It targets generating Apache parquet data files for use with dataframe libraries like polars and pandas in Python. It uses polars (as well as several other extremely useful crates) to transform binary archives (`run_<number>.tar.gz` format) of CAEN CoMPASS data into parquet dataframe files.

spsevb aims to support MacOS, Windows, and (most) Linux.

spsevb is still under heavy development and breaking changes may occur.

## Download and Install

First, you'll need to install the Rust toolchain (compiler, cargo, etc). Go to the [Rust website](https://www.rust-lang.org/tools/install) and follow the instructions there.

Once you have the toolchain, clone the spsevb repostiory using `git clone https://github.com/gwm17/spsevb.git`. Enter the spsevb repostiory and run the command `cargo run`. This will build and launch the spsevb executable.

Currently, spsevb defaults to building the debug executable. Release can be built using the command `cargo -r run` or `cargo --release run`. Once the project reaches a more stable state, release will be set as the default build.

### Python dependencies

Since spsevb is oriented around converting data to a format easily analyzed with Python, it also contains some basic utilities as well as an example script for working with data in Python. To install the necessary dependencies (matplotlib, polars, numpy, etc.) use the requirements.txt in the repository with pip (`pip install -r requirements.txt`). It is highly recommended to use a virtual environment for installing the dependencies (see [here](https://packaging.python.org/en/latest/guides/installing-using-pip-and-virtual-environments/) for details on installing and using virtual environments with pip).

## Use

Enter in the information in the UI and then use the Run button.

spsevb asks the user to define a workspace. The workspace is a parent directory containing all of the relevant subdirectories for event building. When a workspace is chosen, spsevb will check to see if a) the workspace directory exists and b) if the workspace directory contains all necessary subdirectories. It will then create directories as needed (including the parent workspace directory). CoMPASS data archives (`run_<number>.tar.gz` format) should be stored in the `raw_binary` directory of the workspace. Output files (the parquet dataframe files and scaler output files) will be written to the `built` directory of the workspace.

Some important overarching notes:

- spsevb works on a run-by-run basis. That is you can specify a range of runs to event build in the UI, and spsevb will event-build and generate an output for each *individual* run. Merging runs can then be handled after the fact either through python or with a separate Rust app.

- spsevb unpacks the binary archives to the `temp_binary` directory of the workspace using the flate2 and tar crates. spsevb tries to make sure that this temporary unpacked data is always cleaned up after each run. However, in the event of a crash, sometimes `temp_binary` is not cleared. When this happens, it is a good idea to go and manually remove all binary files from `temp_binary`. spsevb should clear the directory when it starts back up, but the consequences of event building with an uncleared `temp_binary` can be severe, often making the output data illegible. Better safe than sorry.

- Make sure that you have permission to read and write to the workspace.

### Event building and the Coincidence Window

The core of event building revolves around the idea of a coincidence window. The coincidence window defines the length of time for which, after an initial detector hit, other detector hits are considered to have come from the same physics event. For spsevb, this is defined by a single user-defined value in nanoseconds, held constant for the entire event building process. spsevb uses an event building architecture similar to the [BoxScore](https://www.sciencedirect.com/science/article/abs/pii/S0168900222001954) model. The main difference is the inital sorting process: rather that using software sorting on arbitrarily buffered data, spsevb relies on the knowledge that CoMPASS saves data from each individual channel in each digitizer to its own file, and that the data in these files is already sorted in time. In a sense, CoMPASS has already done the hard work by pre-sorting so much of the data. This way, spsevb never needs to sort large data buffers, and can run a very basic modified insertion sort efficiently by merely sorting the earliest hit in time from each binary file.

A typical default value for the coincidence window is 3000 ns.

### Channel Map and Dataframe-ing

To use spsevb, there is one key component a user must create: a channel map file. The channel map provides spsevb with information linking the CAEN digitizer board/channel numbers to detector types. An example is included in the etc directory (named ChannelMap.txt). The channel map file is a three-column, whitespace delineated text file. Each row is a single channel in the entire digitizer chain. The first column indicates the board number, the second column indicates the channel number on that board, and the third column is the name of the detector component. Valid detector component names can be found in the source code in src/evb/channel_map.rs. The enum SPSChannelType has a variant for each allowed component. The variant names are the allowed component names (spelled and capitalized exactly as found in the variants). Adding new components is as simple as adding more variants to the SPSChannelType enum; all of the stringificantion and vectorization is handled by the awesome strum crate.

These channel map ids are used to link a data from a given channel to a detector component. These channel map ids are then used to generate the data fields stored in the final dataframe product. This process can be found in the source code at src/evb/sps_data.rs. There are two key components to converting to dataframe relevant structures. One is the SPSDataField enum; each variant of this enum defines one single column in the dataframe. As with the SPSChannelType enum, adding a new column is as simple as adding a new variant to SPSDataField; strum handles everything else. The other aspect is the SPSData struct. SPSData behaves much like a dictionary in Python. It contains a map of SPSDataField variants to a single 64-bit floating point value. The `new` function implemented for SPSData takes in a vector of CoMPASS data and then assigns it to an SPSDataField. This is handled by a single match statement, handling each variant of the channel map. Often times these raw detector components have three associated values (energy, energy short, and timestamp). There can also be "physics" fields, fields which are calculated using raw detector data (examples of this would be x1, x2, and xavg). These do not have an associated channel map, but are rather calculated after all raw data has been handled by checking to see if the SPSData object has identified good data from the appropriate detectors components.

### Scalers and the Scaler list

Sometimes, there are channels which contain data that should not be event built, but rather are just used as raw counting measures. A common example in the SPS setup is the beam integrator. These are commonly referred to as scalers and have to be handled slightly differently than regular data. To declare a channel a scaler, it must be added to the scaler list. The scaler list is a two column, whitespace delineated text file. The first column is the "file pattern". Since the scalers need to be declared before the event building process starts (i.e. before files are read), we cannot use the same board channel scheme used for the channel map, because CoMPASS does not name files using board numbers (which is annoying, but probably a good thing). Instead, CoMPASS names files by board serial number and channel. To that end, the file pattern is `Data_CH<channel_number>@<board_type>_<board_serial_number>`, where the fields in angle brackets should be filled out with the specific information for the scaler. The second column of the scaler list is a name for the scaler.

When a scaler is declared, spsevb removes that binary file from the list of files to event-build, and then counts the number of hits within the file. spsevb then generates a scaler output file along side the dataframe file.

### Kinematics

In brief, a first order correction to kinematic broadening of states can be done by shifting the focal plane upstream or downstream. spsevb can calculate this shift for a given reaction, specified by the target, projectile, and ejectile nuclei as well as the projectile (beam) kinetic energy, SPS (reaction) angle, and SPS magnetic field. spsevb uses this shift to calculate "weights" to apply to the data from the front and back delay lines. The weights are factors equivalent to finding the solution of tracing the particle trajectory to the shifted focal plane. For more information, see the papers by H. Enge on the Enge splipole designs.

In spsevb, nuclei are specified by Z, A. The residual is calculated from the other nuclei. Beam kinetic energy is given in MeV, angle in degrees, and magnetic field in kG (Tesla). Nuclear data is retrieved from the [AMDC](https://www-nds.iaea.org/amdc/) 2016 mass file distributed with the repository. Since the program has the path to the AMDC file hardcoded, always run from the top level of the repository.

The Set button of the kinematics section should be renamed. It does not set values, merely sets the reaction equation.

### Memory Usage and Max Buffer Size

Once data is event built, it is stored in a map like structure which is stored on the heap until converted to a dataframe and written to disk. This does mean that spsevb will need to store the entire dataset in memory (a buffer) until it is written to disk. In general this is a benefit; all file writing occurs at once, which allows the event building to proceed as quickly as possible. However, this can mean that once progress has reached 100%, the progress may "freeze" for a second before allowing a new run command, as writing data to disk can take some time.

As a precaution against extremely large single run datasets, spsevb has a limit on the maximum size of a buffer as 8GB by default. Once the limit is reached, spsevb will stop event building, convert the data and write to disk, and then resume event building. When this fragmentation happens, the spsevb will append a fragment number to the output file name (i.e. `run_<run_num>_<frag_num>.parquet`). These fragment files can be combined later if needed (though in general this is not recommended). Most SPS experiments should never reach this limit, but it is a necessary precaution. This limit may need to be adjusted depending on the hardware used (the max buffer size should not exceed system memory).

Currently max file size is defined in `src/evb/compass_run.rs` as a constant. Eventually this will be promoted to an user input in the GUI.

### Configuration saving

The File menu has options for saving and loading configurations. Configurations are stored as YAML files (using the serde and serde_yaml crates), which are human readable and editable.

## Scripting analysis

See the scripts directory for examples and to access the evbutils module, which contains a working module of useful tools for analysis

### evbutils

    - Cut2D & CutHandler: implementation of 2D graphical cuts (gates) for data analysis. Similar in functionality to ROOT's TCutG object. Requires matplotlib and numpy. Brief description of use: CutHandler should be used as a callback handler for some matplotlib Selector object. CutHandler recieves data on completion of the Selector object, and transforms that data into a Cut2D. Cut2D is a named wrapper around the matplotlib Path object (which in turn is a representation of a path (optionally closed)). Cut2D has functionality to check if a point, list of points, or set of ArrayLike lists of x, y coordinates is inside it's Path. The latter option is probably the most powerful, as it can then be used inside things like polars filter expressions. Cut2D objects can be serialized and deserialzed to JSON. See the source code for examples in scripts/evbutils/cut.py.
