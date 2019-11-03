# hw
## a hackathon homework planner cli

#### ./hw add <name> <date(d/m/y)> -d <description> -c <subject name>
adds an event with a name, date, and optional description and subject name

#### ./hw remove <name>
tries to find an event conataining name, and removes it. Prompts the user if there is more than one possibility

#### ./hw info <name>
tries to find an event containing name, and get's it's info. Prompts the user if there is more than one possibility

#### ./hw ls
Lists all events sorted by date, and color coded by subject

Edit colors_db for colors for subjects as they are added. Uses ansii escape codes


Built in a hackathon!