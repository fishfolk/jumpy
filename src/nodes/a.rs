





  let middle_point = (player_pos + foe_pos) / 2.;
  let mut size = (player_pos - foe_pos).abs();

  // if we'd use scaled X dimension as new Y dimension
  // will it fit the original Y dimension?
  if scale.y < scale.x / aspect  {
      scale.y = scale.x / aspect;
  }
  // if not - lets stretch another axis
  else {
      scale.x = scale.y * aspect;
  }

  Camera2D {
      target: middle_point,
      zoom: vec2(1., -1.) / scale * 2., 
      ..Default::default()
  }

