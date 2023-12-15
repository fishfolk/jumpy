local Entities = s"Entities"
local MapElementHydrated = s"MapElementHydrated"
local ElementHandle = s"ElementHandle"
local BlunderbassMeta = s"BlunderbassMeta"
local AtlasSprite = s"AtlasSprite"
local Sprite = s"Sprite"
local Item = s"Item"
local ItemThrow = s"ItemThrow"
local ItemGrab = s"ItemGrab"
local DehydrateOutOfBounds = s"DehydrateOutOfBounds"
local KinematicBody = s"KinematicBody"
local Transform = s"Transform"
local DropItem = s"DropItem"
local ItemUsed = s"ItemUsed"
local Blunderbass = s"Blunderbass"
local DamageRegion = s"DamageRegion"
local DamageRegionOwner = s"DamageRegionOwner"
local Time = s"Time"
local GlobalRng = s"GlobalRng"
local Lifetime = s"Lifetime"
local BulletHandle = s"BulletHandle"
local Bullet = s"Bullet"
local Vec2 = s"Vec2"

local function hydrate()
  local entities = resources:get(Entities)
  
  for spawner_ent, element_handle in entities:iter_with(ElementHandle, MapElementHydrated:without()) do
    local element = assets:get(element_handle[0])
    local blunderbass_meta = assets:get(element.data)

    if schema_of(blunderbass_meta) == BlunderbassMeta then
      -- Spawn a blunderbass
      local ent = entities:create()
      local sprite = Sprite:create();
      sprite.image = blunderbass_meta.sprite
      components:insert(ent, element_handle)
      components:insert(ent, MapElementHydrated:create())
      components:insert(ent, Blunderbass:create())
      components:insert(ent, sprite)
      components:insert(ent, components:get(spawner_ent, Transform))
      components:insert(ent, Item:create())
      local item_grab = ItemGrab:create()
      item_grab.fin_anim = blunderbass_meta.fin_anim
      item_grab.grab_offset = blunderbass_meta.grab_offset
      components:insert(ent, item_grab)
      components:insert(ent, ItemThrow:create())
      local dehydrate_out_of_bounds = DehydrateOutOfBounds:create()
      dehydrate_out_of_bounds[0] = spawner_ent
      components:insert(ent, dehydrate_out_of_bounds)
      local body = KinematicBody:create()
      body.gravity = assets.root.core.physics.gravity
      body.has_mass = true
      body.has_friction = true
      body.bounciness = 0
      components:insert(ent, body)

      -- Mark spawner as hydrated
      components:insert(spawner_ent, MapElementHydrated:create())
    end
  end
end

local function update()
  local entities = resources:get(Entities)
  local time = resources:get(Time)
  local rng = resources:get(GlobalRng)

  for ent, blunderbass in entities:iter_with(Blunderbass) do
    if blunderbass.cooldown > 0 then
        blunderbass.cooldown = blunderbass.cooldown - time.delta_seconds
    end
  
    local element_handle = components:get(ent, ElementHandle)
    local element = assets:get(element_handle[0])
    local blunderbass_meta = assets:get(element.data)

    local used = components:get(ent, ItemUsed)
    if used then
        if blunderbass.cooldown <= 0 then
            local player_ent = used.owner
            local player_sprite = components:get(player_ent, AtlasSprite)
            local player_transform = components:get(player_ent, Transform)
            local player_body = components:get(player_ent, KinematicBody)
            
            if player_sprite.flip_x then
                player_body.velocity.x = player_body.velocity.x + blunderbass_meta.kickback
            else
                player_body.velocity.x = player_body.velocity.x - blunderbass_meta.kickback
            end
            
            -- Spawn bullets
            for i = 1, blunderbass_meta.bullet_count do
                local bullet_ent = entities:create()
                components:insert(bullet_ent, player_transform)

                local bullet_sprite = Sprite:create()
                bullet_sprite.image = blunderbass_meta.bullet_sprite
                components:insert(bullet_ent, bullet_sprite)

                local bullet_handle = BulletHandle:create()
                bullet_handle[0] = blunderbass_meta.bullet
                components:insert(bullet_ent, bullet_handle)
            
                local bullet = Bullet:create()
                bullet.owner = player_ent
                local direction = Vec2:create()
                if player_sprite.flip_x then
                    direction.x = -1
                else
                    direction.x = 1
                end
                direction.y = (rng:f32() - 0.5) * blunderbass_meta.bullet_spread
                bullet.direction = direction
                components:insert(bullet_ent, bullet)
            end
            
            blunderbass.cooldown = blunderbass_meta.cooldown
        end
        
        components:remove(ent, ItemUsed)
    end
  end
end

session:add_system_to_stage(CoreStage.PreUpdate, hydrate)
session:add_system_to_stage(CoreStage.PostUpdate, update)
