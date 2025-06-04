# Grim Dawn loot cli
GDLC is a simple[1] command line tool to list and search items from Grim
Dawn characters. It reads through the user's stash, save and database files on
each invocation, and doesn't maintain any database or state of its own. The
tool is read-only and doesn't modify any files on disk.

When invoked it lists all items across all characters. When provided with an
argument, it uses it as a filter.

[1] Decrypting the player's save files and cross-referencing the data with the
database & localization files was anything but simple.

![A screenshot of invoking "time ./gdlc.exe demonic b", which listed multiple matches across stashes and inventories. The command completed in 0.66s](/screenshot.png)

# Usage
* The installation location and save directory need to be configured.
    - For Windows: `%UserProfile%\.gdlc.conf`
    - For Linux & others: `~/.config/gdlc/gdlc.conf`
Example config:
```
installation_dir=C:\Games\Grim Dawn\
save_dir=C:\Users\<username>\My Documents\My Games\Grim Dawn\save\
```
Note that values should be without quotes and that variables are not expanded.
The string is simply split on the first '='.

# Bugs & error handling
This tool is quick and dirty. The code has some cruft from figuring it all out.
GDLC expects files to adhere to certain formats, and might crash noisily
if it encounters unexpected data. Game updates that modify the file formats
will break this tool.

On a positive development note, the only external dependency is lz4.

# Credits
I used several other Grim Dawn tools as examples for the stash/character/database logic.
- marius00 for [Grim Dawn Item Assistant](https://github.com/marius00/iagd/).
- Aaron Hutchinson for [Grim Dawn Save Decryption](https://github.com/AaronHutchinson/Grim-Dawn-Save-Decryption/). 
The code samples were especially concise and readable.
- Chris Elison for [GDParser](https://github.com/ChrisElison/GDParser/).
