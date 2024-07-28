(function() {var type_impls = {
"jumpy":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Reflection%3CT,+Const%3CD%3E,+S%3E\" class=\"impl\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#15\">source</a><a href=\"#impl-Reflection%3CT,+Const%3CD%3E,+S%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T, S, const D: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.usize.html\">usize</a>&gt; <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Reflection\">Reflection</a>&lt;T, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;D&gt;, S&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.ComplexField.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::ComplexField\">ComplexField</a>,\n    S: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Storage.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Storage\">Storage</a>&lt;T, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;D&gt;&gt;,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new_containing_point\" class=\"method\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#18\">source</a><h4 class=\"code-header\">pub fn <a href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html#tymethod.new_containing_point\" class=\"fn\">new_containing_point</a>(\n    axis: <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Unit.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Unit\">Unit</a>&lt;<a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;D&gt;, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;1&gt;, S&gt;&gt;,\n    pt: &amp;<a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.OPoint.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::OPoint\">OPoint</a>&lt;T, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;D&gt;&gt;\n) -&gt; <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Reflection\">Reflection</a>&lt;T, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;D&gt;, S&gt;</h4></section></summary><div class=\"docblock\"><p>Creates a new reflection wrt. the plane orthogonal to the given axis and that contains the\npoint <code>pt</code>.</p>\n</div></details></div></details>",0,"jumpy::core::physics::rapier::nalgebra::Reflection1","jumpy::core::physics::rapier::nalgebra::Reflection2","jumpy::core::physics::rapier::nalgebra::Reflection3","jumpy::core::physics::rapier::nalgebra::Reflection4","jumpy::core::physics::rapier::nalgebra::Reflection5","jumpy::core::physics::rapier::nalgebra::Reflection6"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Reflection%3CT,+D,+S%3E\" class=\"impl\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#24\">source</a><a href=\"#impl-Reflection%3CT,+D,+S%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T, D, S&gt; <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Reflection\">Reflection</a>&lt;T, D, S&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.ComplexField.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::ComplexField\">ComplexField</a>,\n    D: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    S: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Storage.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Storage\">Storage</a>&lt;T, D&gt;,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#29\">source</a><h4 class=\"code-header\">pub fn <a href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html#tymethod.new\" class=\"fn\">new</a>(\n    axis: <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Unit.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Unit\">Unit</a>&lt;<a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, D, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;1&gt;, S&gt;&gt;,\n    bias: T\n) -&gt; <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Reflection\">Reflection</a>&lt;T, D, S&gt;</h4></section></summary><div class=\"docblock\"><p>Creates a new reflection wrt. the plane orthogonal to the given axis and bias.</p>\n<p>The bias is the position of the plane on the axis. In particular, a bias equal to zero\nrepresents a plane that passes through the origin.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.axis\" class=\"method\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#38\">source</a><h4 class=\"code-header\">pub fn <a href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html#tymethod.axis\" class=\"fn\">axis</a>(&amp;self) -&gt; &amp;<a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, D, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;1&gt;, S&gt;</h4></section></summary><div class=\"docblock\"><p>The reflection axis.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.bias\" class=\"method\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#47\">source</a><h4 class=\"code-header\">pub fn <a href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html#tymethod.bias\" class=\"fn\">bias</a>(&amp;self) -&gt; T</h4></section></summary><div class=\"docblock\"><p>The reflection bias.</p>\n<p>The bias is the position of the plane on the axis. In particular, a bias equal to zero\nrepresents a plane that passes through the origin.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.reflect\" class=\"method\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#53-56\">source</a><h4 class=\"code-header\">pub fn <a href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html#tymethod.reflect\" class=\"fn\">reflect</a>&lt;R2, C2, S2&gt;(&amp;self, rhs: &amp;mut <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, R2, C2, S2&gt;)<div class=\"where\">where\n    R2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    C2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    S2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.StorageMut.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::StorageMut\">StorageMut</a>&lt;T, R2, C2&gt;,\n    <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/struct.ShapeConstraint.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::base::constraint::ShapeConstraint\">ShapeConstraint</a>: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/trait.SameNumberOfRows.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::base::constraint::SameNumberOfRows\">SameNumberOfRows</a>&lt;R2, D&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Applies the reflection to the columns of <code>rhs</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.reflect_with_sign\" class=\"method\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#70-73\">source</a><h4 class=\"code-header\">pub fn <a href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html#tymethod.reflect_with_sign\" class=\"fn\">reflect_with_sign</a>&lt;R2, C2, S2&gt;(\n    &amp;self,\n    rhs: &amp;mut <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, R2, C2, S2&gt;,\n    sign: T\n)<div class=\"where\">where\n    R2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    C2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    S2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.StorageMut.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::StorageMut\">StorageMut</a>&lt;T, R2, C2&gt;,\n    <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/struct.ShapeConstraint.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::base::constraint::ShapeConstraint\">ShapeConstraint</a>: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/trait.SameNumberOfRows.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::base::constraint::SameNumberOfRows\">SameNumberOfRows</a>&lt;R2, D&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Applies the reflection to the columns of <code>rhs</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.reflect_rows\" class=\"method\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#86-93\">source</a><h4 class=\"code-header\">pub fn <a href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html#tymethod.reflect_rows\" class=\"fn\">reflect_rows</a>&lt;R2, C2, S2, S3&gt;(\n    &amp;self,\n    lhs: &amp;mut <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, R2, C2, S2&gt;,\n    work: &amp;mut <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, R2, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;1&gt;, S3&gt;\n)<div class=\"where\">where\n    R2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    C2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    S2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.StorageMut.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::StorageMut\">StorageMut</a>&lt;T, R2, C2&gt;,\n    S3: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.StorageMut.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::StorageMut\">StorageMut</a>&lt;T, R2&gt;,\n    <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/struct.ShapeConstraint.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::base::constraint::ShapeConstraint\">ShapeConstraint</a>: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/trait.DimEq.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::base::constraint::DimEq\">DimEq</a>&lt;C2, D&gt; + <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/trait.AreMultipliable.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::base::constraint::AreMultipliable\">AreMultipliable</a>&lt;R2, C2, D, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;1&gt;&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Applies the reflection to the rows of <code>lhs</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.reflect_rows_with_sign\" class=\"method\"><a class=\"src rightside\" href=\"https://docs.rs/nalgebra/0.25.0/src/nalgebra/geometry/reflection.rs.html#106-114\">source</a><h4 class=\"code-header\">pub fn <a href=\"jumpy/core/physics/rapier/nalgebra/struct.Reflection.html#tymethod.reflect_rows_with_sign\" class=\"fn\">reflect_rows_with_sign</a>&lt;R2, C2, S2, S3&gt;(\n    &amp;self,\n    lhs: &amp;mut <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, R2, C2, S2&gt;,\n    work: &amp;mut <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Matrix.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Matrix\">Matrix</a>&lt;T, R2, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;1&gt;, S3&gt;,\n    sign: T\n)<div class=\"where\">where\n    R2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    C2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.Dim.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::Dim\">Dim</a>,\n    S2: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.StorageMut.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::StorageMut\">StorageMut</a>&lt;T, R2, C2&gt;,\n    S3: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/trait.StorageMut.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::StorageMut\">StorageMut</a>&lt;T, R2&gt;,\n    <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/struct.ShapeConstraint.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::base::constraint::ShapeConstraint\">ShapeConstraint</a>: <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/trait.DimEq.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::base::constraint::DimEq\">DimEq</a>&lt;C2, D&gt; + <a class=\"trait\" href=\"jumpy/core/physics/rapier/nalgebra/base/constraint/trait.AreMultipliable.html\" title=\"trait jumpy::core::physics::rapier::nalgebra::base::constraint::AreMultipliable\">AreMultipliable</a>&lt;R2, C2, D, <a class=\"struct\" href=\"jumpy/core/physics/rapier/nalgebra/struct.Const.html\" title=\"struct jumpy::core::physics::rapier::nalgebra::Const\">Const</a>&lt;1&gt;&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Applies the reflection to the rows of <code>lhs</code>.</p>\n</div></details></div></details>",0,"jumpy::core::physics::rapier::nalgebra::Reflection1","jumpy::core::physics::rapier::nalgebra::Reflection2","jumpy::core::physics::rapier::nalgebra::Reflection3","jumpy::core::physics::rapier::nalgebra::Reflection4","jumpy::core::physics::rapier::nalgebra::Reflection5","jumpy::core::physics::rapier::nalgebra::Reflection6"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()