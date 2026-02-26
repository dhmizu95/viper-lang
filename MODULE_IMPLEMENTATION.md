Typing Module Implementation
Tasks
Phase 1: AST & Type System
 Add new 
Type
 variants to 
ast/types.rs
: Literal, Annotated, Optional, Union, Final, ClassVar
 Add TypeAlias statement to 
ast/nodes.rs
Phase 2: Parser
 Extend 
parse_type_annotation
 in 
definitions.rs
 to handle Literal[...], Annotated[...], Optional[T], Union[A, B], Final[T], generic bracket notation Name[T]
 Add import-time alias resolution: when from typing import Literal, register Literal as a known type constructor
Phase 3: Module Resolution
 Create src/typing_module.rs — a static module descriptor listing all exported names from typing
 In the type_checker/driver, handle from typing import X by populating the known typing symbols
Phase 4: Type Checker
 In 
semantic/type_checker.rs
, handle FromImport for "typing" module
 Validate Literal[v] — values must be literal constants
 Annotated[T, meta] — pass-through the base type T, store metadata
 Optional[T] — desugar to Union[T, None]
 Enforce Final immutability
Phase 5: Tests
 Write tests/test_typing_basic.vp: from typing import Literal, Annotated
 Write tests/test_typing_optional.vp: Optional[str]
 Write tests/test_typing_union.vp: Union[i64, str]