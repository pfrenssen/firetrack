// Bootstrap alert component. Ref. https://getbootstrap.com/docs/4.0/components/alerts/
#[derive(Serialize, Deserialize)]
pub struct Alert {
    pub alert_type: AlertType,
    pub message: String,
}

// Types of Bootstrap alerts.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
    Info,
    Light,
    Dark,
}
