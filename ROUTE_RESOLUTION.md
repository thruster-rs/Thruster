# Route Resolution Algorithm

The route resolution algorithm should be fairly clear. It goes like this:

### Construction
1. Remove query params, attach to the request object (or somewhere accessible for later)
1. Break a route down by slashes, you now have a list of components
1. Add into a tree, treating `*` and `:param` as wild cards
ex. `get("/a/b")` and `get(":id")` should make a tree like this

```
root (/)
|\
| \
a  *
|
|
b
```

### Resolution
1. Split the incoming route in the same way as during construction
1. Set `current_node = root`
1. If `split.get(0)` does not exist, you're done, return the accumulated middleware
1. Check if the first piece (`split.get(0)`) matches `current_node`
    1. If it does
        1. Set `current_node = child`
        1. Accumulate any middleware attached to that node.
    1. If it does not
        1. Check if there is a wildcard node, if so then `current_node = current_node.wildcard`
        1. If no wildcard, then the route isn't found, call the 404 handler.
1. Remove the first piece of the split (or increment index)
1. Go back to 3
