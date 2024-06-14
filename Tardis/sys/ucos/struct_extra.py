
# Python program to read
# json file
 
 
import json
 
# Opening JSON file
f = open('structs.json')
 
# returns JSON object as 
data = json.load(f)
sets = set()
for i in data:
    sets.add(data[i][0][1])
    # print()
for i in sets:
    print(i.lower() + ", &");
    #    print(i + " " + i.lower())
# Closing file
f.close()
