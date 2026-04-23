local Entities = s"Entities"
local MapElementHydrated = s"MapElementHydrated"
local ElementHandle = s"ElementHandle"
local AnchorMeta = s"AnchorMeta"
local AtlasSprite = s"AtlasSprite"
local Item = s"Item"
local ItemThrow = s"ItemThrow"
local ItemGrab = s"ItemGrab"
local DehydrateOutOfBounds = s"DehydrateOutOfBounds"
local KinematicBody = s"KinematicBody"
local Transform = s"Transform"
local DropItem = s"DropItem"
local ItemUsed = s"ItemUsed"
local IdleAnchor = s"IdleAnchor"
local FallingAnchor = s"FallingAnchor"
local DamageRegion = s"DamageRegion"
local DamageRegionOwner = s"DamageRegionOwner"

local function hydrate()
  local entities = resources:get(Entities)
  
  for spawner_ent, element_handle in entities:iter_with(ElementHandle, MapElementHydrated:without()) do
    local element = assets:get(element_handle[0])
    local anchor_meta = assets:get(element.data)

    if schema_of(anchor_meta) == AnchorMeta then
      -- Spawn an anchor
      local ent = entities:create()
      local sprite = AtlasSprite:create();
      sprite.atlas = anchor_meta.atlas
      components:insert(ent, element_handle)
      components:insert(ent, MapElementHydrated:create())
      components:insert(ent, IdleAnchor:create())
      components:insert(ent, sprite)
      components:insert(ent, components:get(spawner_ent, Transform))
      components:insert(ent, Item:create())
      local item_grab = ItemGrab:create()
      item_grab.fin_anim = anchor_meta.fin_anim
      item_grab.grab_offset = anchor_meta.grab_offset
      components:insert(ent, item_grab)
      components:insert(ent, ItemThrow:create())
      local dehydrate_out_of_bounds = DehydrateOutOfBounds:create()
      dehydrate_out_of_bounds[0] = spawner_ent
      components:insert(ent, dehydrate_out_of_bounds)
      local body = KinematicBody:create()
      -- TODO: Set the body shape and size. Doesn't work in lua yet because
      -- the body shape is an enum.
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

  for ent in entities:iter_with(IdleAnchor) do
    local element_handle = components:get(ent, ElementHandle)
    local element = assets:get(element_handle[0])
    local anchor_meta = assets:get(element.data)

    local used = components:get(ent, ItemUsed)
    if used then
      -- components:remove(ent, ItemUsed)
      components:remove(ent, KinematicBody)
      components:remove(ent, IdleAnchor)
      components:insert(ent, FallingAnchor:create())
      components:insert(ent, DropItem:create())
      local damage = DamageRegion:create()
      damage.size = anchor_meta.body_size
      components:insert(ent, damage)
      local damageOwner = DamageRegionOwner:create()
      damageOwner[0] = used.owner
      components:insert(ent, damageOwner)
    end
  end

  for ent in entities:iter_with(FallingAnchor) do
    local element_handle = components:get(ent, ElementHandle)
    local element = assets:get(element_handle[0])
    local anchor_meta = assets:get(element.data)

    local trans = components:get(ent, Transform)
    trans.translation.y = trans.translation.y - anchor_meta.fall_speed
  end
end

session:add_system_to_stage(CoreStage.PreUpdate, hydrate)
session:add_system_to_stage(CoreStage.PostUpdate, update)
