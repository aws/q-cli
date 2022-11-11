import sys

parts = sys.argv[1].split('-', 2)
parts.append('1')

print(parts[0])
print(parts[1])
