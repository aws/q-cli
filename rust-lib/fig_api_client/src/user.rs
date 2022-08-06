use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub email: String,
    pub id: u64,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum TeamRole {
    Member,
    Admin,
    Owner,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum CustomerStatus {
    #[default]
    Inactive,
    Failed,
    Active,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum CustomerPlan {
    #[default]
    Free,
    Pro,
    Enterprise,
}

impl CustomerPlan {
    /// Checks if a user has access to pro features (Pro and Enterprise)
    pub fn is_pro(&self) -> bool {
        matches!(self, CustomerPlan::Pro | CustomerPlan::Enterprise)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub free_trial_elapsed: bool,
    pub customer_plan: CustomerPlan,
    pub customer_status: CustomerStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamPlan {
    pub id: u64,
    pub name: String,
    pub role: TeamRole,
    #[serde(flatten)]
    pub plan: Plan,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPlan {
    pub id: u64,
    pub email: String,
    pub username: Option<String>,
    #[serde(flatten)]
    pub plan: Plan,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plans {
    pub user_plan: UserPlan,
    pub team_plans: Vec<TeamPlan>,
}

impl Plans {
    pub fn highest_plan(&self) -> CustomerPlan {
        self.user_plan.plan.customer_plan.max(
            self.team_plans
                .iter()
                .map(|plan| plan.plan.customer_plan)
                .max()
                .unwrap_or_default(),
        )
    }
}

pub async fn account() -> fig_request::Result<Account> {
    fig_request::Request::get("/user/account").auth().deser_json().await
}

pub async fn plans() -> fig_request::Result<Plans> {
    fig_request::Request::get("/user/plan").auth().deser_json().await
}
