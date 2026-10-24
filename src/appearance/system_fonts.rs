//! Resolve installed faces and their real variable-font coordinates through the OS.
use crate::i18n;
use crate::storage::Result;
use eframe::egui::FontData;
use eframe::egui::epaint::text::VariationCoords;
use fontique::Attributes;
use fontique::Collection;
use fontique::CollectionOptions;
use fontique::FontWeight;
use fontique::GenericFamily;
use fontique::QueryFamily;
use fontique::QueryStatus;
use fontique::SourceCache;

pub(super) struct SystemFonts {
    collection: Collection,
    pub cache: SourceCache,
    pub names: Vec<String>,
}

impl SystemFonts {
    pub fn new() -> Self {
        let mut collection = Collection::new(CollectionOptions::default());
        let mut names: Vec<String> = collection.family_names().map(str::to_owned).collect();
        names.sort_by_cached_key(|name| name.to_lowercase());
        names.dedup();
        Self {
            collection,
            cache: SourceCache::default(),
            names,
        }
    }

    fn resolve(&mut self, families: Vec<QueryFamily<'_>>, weight: FontWeight) -> Option<FontData> {
        let mut query = self.collection.query(&mut self.cache);
        query.set_families(families);
        query.set_attributes(Attributes {
            weight,
            ..Default::default()
        });
        let mut data = None;
        query.matches_with(|font| {
            let (blob, _) = font.blob.clone().into_raw_parts();
            let mut face = FontData::from_blob(blob, font.index);
            face.tweak.coords =
                VariationCoords::new(font.synthesis.variation_settings().iter().copied());
            data = Some(face);
            QueryStatus::Stop
        });
        data
    }

    pub fn ui(&mut self, weight: FontWeight) -> Option<FontData> {
        let mut families = vec![
            QueryFamily::Named("SF Pro"),
            QueryFamily::Named("SF Pro Text"),
        ];
        if cfg!(target_os = "macos") {
            families.push(QueryFamily::Generic(GenericFamily::SystemUi));
        }
        self.resolve(families, weight)
    }

    pub fn monospace(&mut self) -> Result<FontData> {
        let name = if cfg!(target_os = "windows") {
            "Consolas"
        } else {
            "Menlo"
        };
        self.resolve(
            vec![
                QueryFamily::Named(name),
                QueryFamily::Generic(GenericFamily::Monospace),
            ],
            FontWeight::NORMAL,
        )
        .ok_or_else(|| {
            i18n::message(|m| &m.error_theme_font)
                .arg("font", name)
                .into()
        })
    }

    pub fn load(&mut self, name: &str) -> Result<FontData> {
        if name.eq_ignore_ascii_case("SF Pro") {
            return self.ui(FontWeight::NORMAL).ok_or_else(|| {
                i18n::message(|m| &m.error_theme_font)
                    .arg("font", name)
                    .into()
            });
        }
        let normalized = name.to_lowercase();
        let name = self
            .names
            .iter()
            .find(|candidate| candidate.to_lowercase() == normalized)
            .cloned()
            .unwrap_or_else(|| name.into());
        self.resolve(vec![QueryFamily::Named(&name)], FontWeight::NORMAL)
            .ok_or_else(|| {
                i18n::message(|m| &m.error_theme_font)
                    .arg("font", name)
                    .into()
            })
    }
}
