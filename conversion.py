with open("input.txt") as f:
    text = f.read()

arr = text.split("\\right),\\left(")
arr[0] = arr[0].split('(')[-1]
arr[-1] = arr[-1].split('\\')[0]

total_x = 0
total_y = 0

for a in arr:
    split = a.split(",")
    total_x += float(split[0])
    total_y += float(split[1])

print(total_x / len(arr))
print(total_y / len(arr))

with open("output.txt", "w") as f:
    f.write(",".join(map(lambda s: "(" + s + ")", arr)))
