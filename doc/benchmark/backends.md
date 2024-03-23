# Backends other than redis

## keydb

keydb is as fast/slow as redis

```shell
# brew install keydb
cd ~/gars
rm dump.rdb
keydb-server

# gars
cd ~/gars
gars env

gars status drop; gars gen genome/genome.fa.gz --piece 500000;

hyperfine --warmup 1 --export-markdown threads.md.tmp \
    -n 'serial' \
    '
    gars clear feature;
    gars feature features/T-DNA.CSHL.rg --tag CSHL;
    gars feature features/T-DNA.FLAG.rg --tag FLAG;
    gars feature features/T-DNA.MX.rg   --tag MX;
    gars feature features/T-DNA.RATM.rg --tag RATM;
    ' \
    -n 'parallel -j 1' \
    '
    gars clear feature;
    parallel -j 1 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 2' \
    '
    gars clear feature;
    parallel -j 2 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    ' \
    -n 'parallel -j 4' \
    '
    gars clear feature;
    parallel -j 4 "
        echo {}
        gars feature features/T-DNA.{}.rg --tag {}
        " ::: CSHL FLAG MX RATM
    '

cat threads.md.tmp

```

### R7 5800 Windows 11 WSL

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 16.082 ± 0.197 |  15.746 |  16.531 | 1.70 ± 0.04 |
| `parallel -j 1` | 16.938 ± 0.246 |  16.335 |  17.178 | 1.80 ± 0.05 |
| `parallel -j 2` |  9.824 ± 0.225 |   9.423 |  10.263 | 1.04 ± 0.03 |
| `parallel -j 4` |  9.435 ± 0.194 |   9.104 |   9.808 |        1.00 |

### i7 8700K macOS Big Sur

| Command         |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------|---------------:|--------:|--------:|------------:|
| `serial`        | 14.911 ± 0.271 |  14.646 |  15.370 | 1.60 ± 0.06 |
| `parallel -j 1` | 15.783 ± 0.486 |  15.088 |  16.607 | 1.69 ± 0.07 |
| `parallel -j 2` | 10.753 ± 0.351 |  10.438 |  11.437 | 1.15 ± 0.05 |
| `parallel -j 4` |  9.342 ± 0.296 |   8.994 |   9.795 |        1.00 |

## dragonfly

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

## garnet

```shell
brew install dotnet nuget

#curl -LO https://github.com/microsoft/garnet/releases/download/v1.0.0/Microsoft.Garnet.1.0.0.nupkg
#winget install --id "Microsoft.DotNet.SDK.8"

cd Scripts
curl -L https://github.com/microsoft/garnet/archive/refs/tags/v1.0.0.tar.gz |
    tar xvz

cd garnet-1.0.0/main/GarnetServer/
dotnet run -c Release -f net8.0 -- --port 6379

```
