
```hs
solveWith minisat $ do
  let n = 40
      -- f ist Liste aller Positionen (wie auf Folie)
      f = (,) <$> [0..n-1] <*> [0..n-1]
  qss <- replicateM n $ replicateM n (exists @Bit)
  let q (x,y) = qss !! x !! y
  assert $ and $ do qs <- qss; return $ or qs
  assert $ and $ do
    a <- f ; b <- f
    guard $ bedroht a b
    return $ not (q a) || not (q b)
  return qss
```