#diffgeo #topology

## Definition

A *chart* or *local parameter representation* $x$ of a [[Manifold]] $M$ is a bijective map $U \to V$ for $U \subseteq M$ and $V \subseteq \mathbb{R}^n$.

We write $(U, x)$ and call the projection $x^i := \mathrm{pr}_i x$ onto the $i$th component as $i$th *coordinate function*.
If $x(p) = 0$, $(U, x)$ is *centered* in $p$.

Two charts $(U, x), (V,y)$ are $C^r$-*related* if $U \cap V = \emptyset$ or $x(U \cap V)$ and $y(U \cap V)$ are open and $x \circ y^{-1}$ and $y \circ x^{-1}$ are $C^r$-diffeomorphic.
In the latter case, $x \circ y^{-1}$ or $y \circ x^{-1}$ is also called a *change of charts*.

## Properties
