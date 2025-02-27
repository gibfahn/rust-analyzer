//! Defines database & queries for name resolution.
use std::sync::Arc;

use base_db::{salsa, CrateId, SourceDatabase, Upcast};
use either::Either;
use hir_expand::{db::AstDatabase, HirFileId};
use la_arena::ArenaMap;
use syntax::{ast, AstPtr, SmolStr};

use crate::{
    adt::{EnumData, StructData},
    attr::{Attrs, AttrsWithOwner},
    body::{scope::ExprScopes, Body, BodySourceMap},
    data::{
        ConstData, FunctionData, ImplData, Macro2Data, MacroRulesData, ProcMacroData, StaticData,
        TraitData, TypeAliasData,
    },
    generics::GenericParams,
    import_map::ImportMap,
    intern::Interned,
    item_tree::ItemTree,
    lang_item::{LangItemTarget, LangItems},
    nameres::DefMap,
    visibility::{self, Visibility},
    AttrDefId, BlockId, BlockLoc, ConstId, ConstLoc, DefWithBodyId, EnumId, EnumLoc, ExternBlockId,
    ExternBlockLoc, FunctionId, FunctionLoc, GenericDefId, ImplId, ImplLoc, LocalEnumVariantId,
    LocalFieldId, Macro2Id, Macro2Loc, MacroRulesId, MacroRulesLoc, ProcMacroId, ProcMacroLoc,
    StaticId, StaticLoc, StructId, StructLoc, TraitId, TraitLoc, TypeAliasId, TypeAliasLoc,
    UnionId, UnionLoc, VariantId,
};

#[salsa::query_group(InternDatabaseStorage)]
pub trait InternDatabase: SourceDatabase {
    #[salsa::interned]
    fn intern_function(&self, loc: FunctionLoc) -> FunctionId;
    #[salsa::interned]
    fn intern_struct(&self, loc: StructLoc) -> StructId;
    #[salsa::interned]
    fn intern_union(&self, loc: UnionLoc) -> UnionId;
    #[salsa::interned]
    fn intern_enum(&self, loc: EnumLoc) -> EnumId;
    #[salsa::interned]
    fn intern_const(&self, loc: ConstLoc) -> ConstId;
    #[salsa::interned]
    fn intern_static(&self, loc: StaticLoc) -> StaticId;
    #[salsa::interned]
    fn intern_trait(&self, loc: TraitLoc) -> TraitId;
    #[salsa::interned]
    fn intern_type_alias(&self, loc: TypeAliasLoc) -> TypeAliasId;
    #[salsa::interned]
    fn intern_impl(&self, loc: ImplLoc) -> ImplId;
    #[salsa::interned]
    fn intern_extern_block(&self, loc: ExternBlockLoc) -> ExternBlockId;
    #[salsa::interned]
    fn intern_block(&self, loc: BlockLoc) -> BlockId;
    #[salsa::interned]
    fn intern_macro2(&self, loc: Macro2Loc) -> Macro2Id;
    #[salsa::interned]
    fn intern_proc_macro(&self, loc: ProcMacroLoc) -> ProcMacroId;
    #[salsa::interned]
    fn intern_macro_rules(&self, loc: MacroRulesLoc) -> MacroRulesId;
}

#[salsa::query_group(DefDatabaseStorage)]
pub trait DefDatabase: InternDatabase + AstDatabase + Upcast<dyn AstDatabase> {
    #[salsa::input]
    fn enable_proc_attr_macros(&self) -> bool;

    #[salsa::invoke(ItemTree::file_item_tree_query)]
    fn file_item_tree(&self, file_id: HirFileId) -> Arc<ItemTree>;

    #[salsa::invoke(crate_def_map_wait)]
    #[salsa::transparent]
    fn crate_def_map(&self, krate: CrateId) -> Arc<DefMap>;

    #[salsa::invoke(DefMap::crate_def_map_query)]
    fn crate_def_map_query(&self, krate: CrateId) -> Arc<DefMap>;

    /// Computes the block-level `DefMap`, returning `None` when `block` doesn't contain any inner
    /// items directly.
    ///
    /// For example:
    ///
    /// ```
    /// fn f() { // (0)
    ///     { // (1)
    ///         fn inner() {}
    ///     }
    /// }
    /// ```
    ///
    /// The `block_def_map` for block 0 would return `None`, while `block_def_map` of block 1 would
    /// return a `DefMap` containing `inner`.
    #[salsa::invoke(DefMap::block_def_map_query)]
    fn block_def_map(&self, block: BlockId) -> Option<Arc<DefMap>>;

    #[salsa::invoke(StructData::struct_data_query)]
    fn struct_data(&self, id: StructId) -> Arc<StructData>;

    #[salsa::invoke(StructData::union_data_query)]
    fn union_data(&self, id: UnionId) -> Arc<StructData>;

    #[salsa::invoke(EnumData::enum_data_query)]
    fn enum_data(&self, e: EnumId) -> Arc<EnumData>;

    #[salsa::invoke(ImplData::impl_data_query)]
    fn impl_data(&self, e: ImplId) -> Arc<ImplData>;

    #[salsa::invoke(TraitData::trait_data_query)]
    fn trait_data(&self, e: TraitId) -> Arc<TraitData>;

    #[salsa::invoke(TypeAliasData::type_alias_data_query)]
    fn type_alias_data(&self, e: TypeAliasId) -> Arc<TypeAliasData>;

    #[salsa::invoke(FunctionData::fn_data_query)]
    fn function_data(&self, func: FunctionId) -> Arc<FunctionData>;

    #[salsa::invoke(ConstData::const_data_query)]
    fn const_data(&self, konst: ConstId) -> Arc<ConstData>;

    #[salsa::invoke(StaticData::static_data_query)]
    fn static_data(&self, konst: StaticId) -> Arc<StaticData>;

    #[salsa::invoke(Macro2Data::macro2_data_query)]
    fn macro2_data(&self, makro: Macro2Id) -> Arc<Macro2Data>;

    #[salsa::invoke(MacroRulesData::macro_rules_data_query)]
    fn macro_rules_data(&self, makro: MacroRulesId) -> Arc<MacroRulesData>;

    #[salsa::invoke(ProcMacroData::proc_macro_data_query)]
    fn proc_macro_data(&self, makro: ProcMacroId) -> Arc<ProcMacroData>;

    #[salsa::invoke(Body::body_with_source_map_query)]
    fn body_with_source_map(&self, def: DefWithBodyId) -> (Arc<Body>, Arc<BodySourceMap>);

    #[salsa::invoke(Body::body_query)]
    fn body(&self, def: DefWithBodyId) -> Arc<Body>;

    #[salsa::invoke(ExprScopes::expr_scopes_query)]
    fn expr_scopes(&self, def: DefWithBodyId) -> Arc<ExprScopes>;

    #[salsa::invoke(GenericParams::generic_params_query)]
    fn generic_params(&self, def: GenericDefId) -> Interned<GenericParams>;

    #[salsa::invoke(Attrs::variants_attrs_query)]
    fn variants_attrs(&self, def: EnumId) -> Arc<ArenaMap<LocalEnumVariantId, Attrs>>;

    #[salsa::invoke(Attrs::fields_attrs_query)]
    fn fields_attrs(&self, def: VariantId) -> Arc<ArenaMap<LocalFieldId, Attrs>>;

    #[salsa::invoke(crate::attr::variants_attrs_source_map)]
    fn variants_attrs_source_map(
        &self,
        def: EnumId,
    ) -> Arc<ArenaMap<LocalEnumVariantId, AstPtr<ast::Variant>>>;

    #[salsa::invoke(crate::attr::fields_attrs_source_map)]
    fn fields_attrs_source_map(
        &self,
        def: VariantId,
    ) -> Arc<ArenaMap<LocalFieldId, Either<AstPtr<ast::TupleField>, AstPtr<ast::RecordField>>>>;

    #[salsa::invoke(AttrsWithOwner::attrs_query)]
    fn attrs(&self, def: AttrDefId) -> AttrsWithOwner;

    #[salsa::invoke(LangItems::crate_lang_items_query)]
    fn crate_lang_items(&self, krate: CrateId) -> Arc<LangItems>;

    #[salsa::invoke(LangItems::lang_item_query)]
    fn lang_item(&self, start_crate: CrateId, item: SmolStr) -> Option<LangItemTarget>;

    #[salsa::invoke(ImportMap::import_map_query)]
    fn import_map(&self, krate: CrateId) -> Arc<ImportMap>;

    #[salsa::invoke(visibility::field_visibilities_query)]
    fn field_visibilities(&self, var: VariantId) -> Arc<ArenaMap<LocalFieldId, Visibility>>;

    // FIXME: unify function_visibility and const_visibility?
    #[salsa::invoke(visibility::function_visibility_query)]
    fn function_visibility(&self, def: FunctionId) -> Visibility;

    #[salsa::invoke(visibility::const_visibility_query)]
    fn const_visibility(&self, def: ConstId) -> Visibility;

    #[salsa::transparent]
    fn crate_limits(&self, crate_id: CrateId) -> CrateLimits;
}

fn crate_def_map_wait(db: &dyn DefDatabase, krate: CrateId) -> Arc<DefMap> {
    let _p = profile::span("crate_def_map:wait");
    db.crate_def_map_query(krate)
}

pub struct CrateLimits {
    /// The maximum depth for potentially infinitely-recursive compile-time operations like macro expansion or auto-dereference.
    pub recursion_limit: u32,
}

fn crate_limits(db: &dyn DefDatabase, crate_id: CrateId) -> CrateLimits {
    let def_map = db.crate_def_map(crate_id);

    CrateLimits {
        // 128 is the default in rustc.
        recursion_limit: def_map.recursion_limit().unwrap_or(128),
    }
}
