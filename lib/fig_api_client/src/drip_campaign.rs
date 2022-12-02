use std::path::PathBuf;

use serde::{
    Deserialize,
    Serialize,
};

static DRIP_SPACING: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 47);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drip {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DateTime(#[serde(with = "time::serde::iso8601")] time::OffsetDateTime);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DripCampaign {
    pub drips: Vec<Drip>,
    pub index: usize,
    pub last_sent: Option<DateTime>,
    pub email: String,
}

impl DripCampaign {
    fn path() -> Result<PathBuf, fig_util::directories::DirectoryError> {
        Ok(fig_util::directories::fig_data_dir()?.join("drip_campaign.json"))
    }

    pub fn load_local() -> Result<Option<Self>, fig_request::Error> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(path)?;
        let campaign: DripCampaign = serde_json::from_str(&contents)?;
        Ok(Some(campaign))
    }

    fn save(&self) -> Result<(), fig_request::Error> {
        let path = Self::path()?;

        // If the folder doesn't exist, create it.
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut file_opts = std::fs::File::options();
        file_opts.create(true).write(true).truncate(true);
        let mut file = file_opts.open(&path)?;
        serde_json::to_writer_pretty(&mut file, self)?;

        Ok(())
    }

    pub async fn load() -> Result<Option<Self>, fig_request::Error> {
        let email = match fig_request::auth::get_email() {
            Some(email) => email,
            None => return Ok(None),
        };

        if let Some(local) = DripCampaign::load_local()? {
            if local.email == email {
                return Ok(Some(local));
            }
        }

        let drips = fig_request::Request::get("/user/drip-campaign")
            .auth()
            .deser_json()
            .await?;

        let result = DripCampaign {
            drips,
            email,
            index: 0,
            last_sent: None,
        };

        result.save()?;

        Ok(Some(result))
    }

    pub fn get_current_message(&self) -> Option<&Drip> {
        let sent_recently = self
            .last_sent
            .as_ref()
            .map(|sent| {
                let DateTime(x) = sent.clone();
                let time_since = time::OffsetDateTime::now_utc() - x;
                time_since < DRIP_SPACING
            })
            .unwrap_or(false);

        if sent_recently {
            return None;
        }

        self.drips.get(self.index)
    }

    pub async fn increment_drip(&mut self) -> Result<(), fig_request::Error> {
        self.index += 1;
        self.last_sent = Some(DateTime(time::OffsetDateTime::now_utc()));

        if self.index >= self.drips.len() {
            fig_request::Request::post("/user/drip-campaign/complete")
                .auth()
                .json()
                .await
                .ok();
        }

        self.save()?;

        Ok(())
    }
}
