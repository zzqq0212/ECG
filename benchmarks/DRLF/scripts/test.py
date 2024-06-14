import math
def dis_to_weight(distance):
    
    atan_value = math.atan(float(distance))
    # print(atan_value)
    normalized_value = (atan_value + math.pi/2) / math.pi 
    # print(normalized_value)
    value = (1 - normalized_value) * 16 + 1
    # print(value)

    hex_value = hex(int(value))[2:]
    # print(hex_value)
    weight = hex_value[0]
    print(weight)

    return weight

a = 1.0
b = 1.1
c = 1.2
d = 1.3
e = 2.0
f = 3.541507861093852
g = 6.476322535734953

A = dis_to_weight(a)
B = dis_to_weight(b)
C = dis_to_weight(c)
D = dis_to_weight(d)
E = dis_to_weight(e)
F = dis_to_weight(f)
G = dis_to_weight(g)

# print(A,B,C,D,E,F,G)



