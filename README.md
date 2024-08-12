This is my first Rust project.

The result is a binary crate that downloads time series data, annotations, and predictions from a [label studio](https://labelstud.io/) instance. Pass in your api token, host name, and comma-separated list of project names as environment variables (or as command line arguments) to the binary, and it will output a "relevant" csv to stdout.

# Parameters
1. `LSRS_API_TOKEN`: label studio api authentication token
2. `LSRS_HOST_NAME`: host name of the label studio instance (expects protocol (HTTP/HTTPS))
3. `LSRS_PROJECT_NAMES`: comma-separated list of case-sensitive project names

The above (before the colon) are the names of the environment variables to set. You can also pass them as command line arguments in the order listed above. Arguments passed as environment variables will take precedence, so if you set any environment variables, make sure to NOT pass it in as a command line argument as well. For example, if you set `LSRS_API_TOKEN="1234..."`, simply omit the api token from your command line list of arguments and continue to pass in the other two in the same order.

# "Relevant"
Currently the script supports outputting the following four columns: 
* `start` (integer)
* `end` (integer)
* `method` ("predicted" | "manual")
* `file_upload` (string)
* `project` (string)
* `labels` (string), bar-seperated list of labels (from `timeserieslabels`)

This can be changed by updating the `get_relevant_data` function, and you can save more data from the api to work with by un-commenting or adding fields in the structs defined after line `116`.

Please reach out to me at patrick@patrickdwyer.com if you have any questions/concerns/suggestions.

P.S. The complexitly in parsing for this project results from quirks in label studio's api which don't often match one's expectations. These quirks may be considered bugs, but in any case, please let me know if the script doesn't work for you as it's likely label studio has updated their api.