fn plot(st:vec2f,pct:f32) -> f32{
  return  smoothstep( pct-0.02, pct, st.y) -
          smoothstep( pct, pct+0.02, st.y);
}
