use cloud_storage::{BackupStorageProvider, LocalBackupProvider};

#[tokio::test]
async fn local_backup_round_trip() {
    let dir = std::env::temp_dir().join(format!("ws-backup-test-{}", uuid::Uuid::new_v4()));
    let provider = LocalBackupProvider::new(&dir);
    let data = b"backup payload";

    provider
        .upload("tenant-1", "bundle.tar.gz", data)
        .await
        .expect("upload");
    let loaded = provider
        .download("tenant-1", "bundle.tar.gz")
        .await
        .expect("download");
    assert_eq!(loaded, data);

    provider
        .delete("tenant-1", "bundle.tar.gz")
        .await
        .expect("delete");
    let _ = std::fs::remove_dir_all(dir);
}
