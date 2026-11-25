fn main() {
    #[cfg(windows)]
    {
        embed_resource::compile(
            "assets/icons/snipster.rc",
            embed_resource::NONE,
        )
        .manifest_required()
        .unwrap();
    }
}
