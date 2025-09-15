# Embassy STM32 example

## Networking

The examples expect a static IP network setup.
On Linux, the interface directly connected to the board requires the IP added via:

```shell
# Replace `enp0s31f6` with the appropriate interface.
sudo ip addr add 192.168.56.25/24 dev enp0s31f6
```
