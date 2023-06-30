# dragonfly

```shell
docker run -p 6379:6379 --ulimit memlock=-1 docker.dragonflydb.io/dragonflydb/dragonfly

redis-cli PING

```

```shell
curl -L https://dragonflydb.gateway.scarf.sh/1.4.0/dragonfly-x86_64.tar.gz |
    tar xvz

mv dragonfly-x86_64 ~/bin/dragonfly

dragonfly --logtostderr

```
