# Required environment variables

| Name               | Description                                       |
| ------------------ | ------------------------------------------------- |
| STRESS_SOCKET      | Path to socket to serve from/send to              |
| STRESS_CLOSE_AFTER | (Server) Close after X connections                |
| STRESS_KIND        | (Client) Type of stress test to run               |
| STRESS_CYCLES      | (Client) Number of cycles to run                  |
| STRESS_SLEEP       | (Client) Micros to sleep between cycles           |
| STRESS_PARALLEL    | (Client) Instances of the test to run in parallel |

# Test kinds

| Name      | Description                                                  |
| --------- | ------------------------------------------------------------ |
| increment | Sends incrementing numbers, erroring if any are out of order |
