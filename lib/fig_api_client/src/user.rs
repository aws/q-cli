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
    pub plan: Option<Plan>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPlan {
    pub id: u64,
    pub email: String,
    pub username: Option<String>,
    #[serde(flatten)]
    pub plan: Option<Plan>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plans {
    pub user_plan: UserPlan,
    pub team_plans: Vec<TeamPlan>,
}

impl Plans {
    pub fn highest_plan(&self) -> CustomerPlan {
        self.user_plan
            .plan
            .as_ref()
            .map(|plan| plan.customer_plan)
            .unwrap_or_default()
            .max(
                self.team_plans
                    .iter()
                    .filter_map(|plan| plan.plan.as_ref().map(|plan| plan.customer_plan))
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: u64,
    pub name: String,
    pub namespace_id: u64,
    pub specs: Vec<String>,
}

pub async fn teams() -> fig_request::Result<Vec<Team>> {
    fig_request::Request::get("/teams").auth().deser_json().await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_user_plan(level: CustomerPlan) -> UserPlan {
        UserPlan {
            id: 0,
            email: String::new(),
            username: None,
            plan: Some(Plan {
                free_trial_elapsed: false,
                customer_plan: level,
                customer_status: CustomerStatus::Active,
            }),
        }
    }

    fn mock_team_plan(level: CustomerPlan) -> TeamPlan {
        TeamPlan {
            id: 0,
            name: String::new(),
            role: TeamRole::Member,
            plan: Some(Plan {
                free_trial_elapsed: false,
                customer_plan: level,
                customer_status: CustomerStatus::Active,
            }),
        }
    }

    #[test]
    fn is_pro() {
        assert!(!CustomerPlan::Free.is_pro());
        assert!(CustomerPlan::Pro.is_pro());
        assert!(CustomerPlan::Enterprise.is_pro());
    }

    #[test]
    fn highest_plan() {
        assert_eq!(
            Plans {
                user_plan: mock_user_plan(CustomerPlan::Free),
                team_plans: vec![mock_team_plan(CustomerPlan::Free), mock_team_plan(CustomerPlan::Pro)]
            }
            .highest_plan(),
            CustomerPlan::Pro
        );
        assert_eq!(
            Plans {
                user_plan: mock_user_plan(CustomerPlan::Enterprise),
                team_plans: vec![mock_team_plan(CustomerPlan::Free), mock_team_plan(CustomerPlan::Pro)]
            }
            .highest_plan(),
            CustomerPlan::Enterprise
        );
    }
}
