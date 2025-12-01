# Algernon-Solver

## Como rodar?

1. clone este repositório:

``` bash
git clone https://github.com/ianpsa/Algernon-Solver/
```
2. rode o `culling games`

``` bash
cd culling_games
colcon build

# para zsh
source install/local_setup.zsh

# para bash
source install/local_setup.bsh

ros2 run cg maze
```

3. Agora que os tópicos estão expostos, no mesmo ambiente você pode rodar:

``` bash
# Encontre a pasta do Algernon-Solver
cd Algernon-Solver

cargo build --release
```

## Modos de execução

O Algernon-Solver possui dois modos de operação:

### 1. Modo Pathfinding (Mapa Completo)
Usa o mapa completo para encontrar o caminho mais curto usando BFS:

``` bash
./target/release/algernon-solver pathfinding
```

### 2. Modo Explorer (Exploração)
Explora o mapa usando apenas sensores e monta o mapa usando wall following + BFS:

``` bash
./target/release/algernon-solver explorer
```

Pronto! veja algernon se contorcendo por entre os obstáculos do labirinto...

## Videozão da massa

[Link do vídeo](https://drive.google.com/file/d/1Pn3JndurPFw6__gpZmwyfoJ9d2JGue60/view?usp=sharing)
