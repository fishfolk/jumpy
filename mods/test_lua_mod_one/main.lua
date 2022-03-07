print("Hello from lua mod!")
require "second_file"
return {
    fixed_update_physics_bodies = function()
        print("Hello from fixed_update_physics_body")
    end
}