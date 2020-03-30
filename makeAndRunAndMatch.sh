bash makeAndRun.sh | grep "A: " > log.out

python3 compareTrace.py | less
