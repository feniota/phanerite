//! Stable, domain-specific keys for accounts.
//!
//! [`AccountIdent`] identifies a registered account without including
//! renewable credentials. It is suitable for lookups in
//! [`crate::auth::MultiAccount`], but is not an authentication token and must
//! not be treated as proof of identity.

/// The authentication provider represented by an [`AccountIdent`].
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum AccountType {
    /// An account authenticated through Microsoft's official Minecraft
    /// services.
    Microsoft,
    /// An account authenticated through a third-party Yggdrasil service.
    Yggdrasil,
    /// A local, unauthenticated offline account.
    Offline,
}

/// A stable key for a registered account.
///
/// The three fields distinguish accounts across providers and authentication
/// services while remaining independent of credentials that can be refreshed
/// or revoked. Construct identifiers through [`crate::auth::Account::identifier`]
/// when an [`crate::auth::Account`] is available.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct AccountIdent {
    /// The account's authentication provider.
    pub acc_type: AccountType,
    /// The specific authentication service within the provider.
    ///
    /// This is `"official"` for Microsoft, `"offline"` for offline accounts,
    /// and the configured server URL for Yggdrasil.
    pub service: String,
    /// The provider-scoped account identity.
    ///
    /// This is the player UUID for offline accounts, XUID for Microsoft
    /// accounts, and username for Yggdrasil accounts.
    pub ident: String,
}
