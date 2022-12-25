use std::fmt::Display;

use iced::{
    alignment::Horizontal,
    executor,
    theme::{self, Palette},
    widget::{
        button, column, container, horizontal_space, pick_list, row, scrollable, text, text_input,
        vertical_space,
    },
    Alignment, Application, Color, Command, Element, Length, Theme,
};
use image::ImageBuffer;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use rfd::FileDialog;

use crate::{
    backup::{create_backup, recover_backup, BackupConfig, BackupShare},
    crypto::Secret,
    passphrase::gen_passphrase,
    printer::print_pdf,
    qrcode::qrcode_decode,
};

pub struct HyperbackedApp {
    page: AppPage,
    secret: String,
    passphrase: String,
    label: String,
    backup_type: BackupType,
    generated_backup: Option<Vec<BackupShare>>,
    should_exit: bool,
    is_scanning: bool,
    scanned_codes: Vec<Vec<u8>>,
    decoded_secret: String,
}

#[derive(Debug, Clone)]
pub enum AppPage {
    Welcome,
    CreateBackup,
    RestoreBackup,
    BackupGenerating,
    BackupResults,
    DecodeSuccess,
    DecodeFailure,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchPage(AppPage),
    SecretChanged(String),
    PassphraseChanged(String),
    GenerateSecret,
    CreateBackup,
    LabelChanged(String),
    BackupTypeChanged(BackupType),
    BackupCompleted(Option<Vec<BackupShare>>),
    SaveBackup(usize),
    End,
    ScanCode,
    ScanComplete(Option<String>),
    DecodeSecrets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupType {
    Standard,
    Distributed { min: u8, max: u8 },
}

impl Default for HyperbackedApp {
    fn default() -> Self {
        Self {
            page: AppPage::Welcome,
            secret: Default::default(),
            passphrase: Default::default(),
            label: Default::default(),
            backup_type: BackupType::Standard,
            generated_backup: None,
            should_exit: false,
            is_scanning: false,
            scanned_codes: Vec::new(),
            decoded_secret: Default::default(),
        }
    }
}

impl Application for HyperbackedApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (HyperbackedApp::default(), Command::none())
    }

    fn theme(&self) -> Theme {
        Theme::custom(Palette {
            background: Color::from_rgb(0.09, 0.09, 0.15),
            text: Color::from_rgb(0.8, 0.8, 0.8),
            primary: Color::from_rgb(0.5, 0.5, 0.75),
            success: Color::from_rgb(0.33, 0.7, 0.5),
            danger: Color::from_rgb(0.90, 0.30, 0.45),
        })
    }

    fn title(&self) -> String {
        String::from("Hyperbacked")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SwitchPage(page) => {
                self.page = page;
            }
            Message::SecretChanged(secret) => {
                self.secret = secret;
            }
            Message::PassphraseChanged(passphrase) => {
                self.passphrase = passphrase;
            }
            Message::GenerateSecret => {
                self.passphrase = gen_passphrase(6);
            }
            Message::CreateBackup => {
                self.page = AppPage::BackupGenerating;

                let backup_type = self.backup_type.clone();
                let secret = self.secret.clone();
                let passphrase = self.passphrase.clone();

                return Command::perform(
                    async move {
                        let secrets = &[Secret {
                            value: secret.as_str(),
                            password: passphrase.as_str(),
                        }];

                        let required_shares = match backup_type {
                            BackupType::Standard => 1,
                            BackupType::Distributed { min, .. } => min,
                        };

                        let num_shares = match backup_type {
                            BackupType::Standard => 1,
                            BackupType::Distributed { max, .. } => max,
                        };

                        return create_backup(
                            secrets.to_vec(),
                            BackupConfig {
                                required_shares,
                                num_shares,
                            },
                        )
                        .ok();
                    },
                    Message::BackupCompleted,
                );
            }
            Message::LabelChanged(label) => {
                self.label = label;
            }
            Message::BackupTypeChanged(backup_type) => {
                self.backup_type = backup_type;
            }
            Message::BackupCompleted(result) => {
                self.generated_backup = result;
                self.page = AppPage::BackupResults;
            }
            Message::SaveBackup(num) => {
                let file = FileDialog::new().add_filter("pdf", &["pdf"]).save_file();
                if let Some(file) = file {
                    let backup = self.generated_backup.as_ref().unwrap();

                    let share = backup
                        .iter()
                        .find(|backup| backup.number == num)
                        .expect("Could not find backup to save");

                    let pdf_data = print_pdf(share, &self.label, backup.len()).unwrap();
                    pdf_data.render_to_file(file).unwrap();
                }
            }
            Message::End => {
                self.should_exit = true;
            }
            Message::ScanCode => {
                self.is_scanning = true;

                return Command::perform(
                    async {
                        let mut camera = Camera::new(
                            CameraIndex::Index(0),
                            RequestedFormat::new::<RgbFormat>(
                                RequestedFormatType::AbsoluteHighestResolution,
                            ),
                        )
                        .unwrap();

                        let decoder = bardecoder::default_decoder();

                        loop {
                            let frame = camera.frame().unwrap();
                            let frame_img = frame.decode_image::<RgbFormat>().unwrap();

                            let raw_img = frame_img.as_raw();

                            let barcodes = decoder.decode(&image::DynamicImage::ImageRgb8(
                                ImageBuffer::from_raw(
                                    frame_img.width(),
                                    frame_img.height(),
                                    raw_img.to_owned(),
                                )
                                .unwrap(),
                            ));

                            let first_barcode = barcodes
                                .iter()
                                .map_while(|barcode| barcode.as_ref().ok())
                                .nth(0);

                            if let Some(barcode) = first_barcode {
                                return Some(barcode.to_owned());
                            }
                        }
                    },
                    Message::ScanComplete,
                );
            }
            Message::ScanComplete(str) => {
                self.is_scanning = false;
                if let Some(str) = str {
                    let data = qrcode_decode(&str);
                    if let Some(data) = data {
                        self.scanned_codes.push(data);
                    } else {
                        println!("Failed to decode!");
                    }
                } else {
                    println!("Failed to scan!");
                }
            }
            Message::DecodeSecrets => {
                let backup_secret = recover_backup(&self.scanned_codes, &self.passphrase);
                match backup_secret {
                    Ok(decoded) => {
                        self.decoded_secret = decoded;
                        self.page = AppPage::DecodeSuccess;
                    }
                    Err(_) => {
                        self.page = AppPage::DecodeFailure;
                    }
                }
            }
        }
        Command::none()
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn view(&self) -> Element<Message> {
        let page = match &self.page {
            AppPage::Welcome => self.welcome_page(),
            AppPage::CreateBackup => self.create_backup_page(),
            AppPage::BackupGenerating => self.generating_page(),
            AppPage::BackupResults => self.backup_results_page(),
            AppPage::RestoreBackup => self.recover_backup_page(),
            AppPage::DecodeSuccess => self.decode_success_page(),
            _ => self.welcome_page(),
        };

        container(page)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(40)
            .center_x()
            .center_y()
            .into()
    }
}

impl HyperbackedApp {
    fn get_passphrase_crack_time(&self) -> String {
        let estimate = zxcvbn::zxcvbn(&self.passphrase, &[]);
        match estimate {
            Ok(estimate) => {
                let crack_time = estimate.crack_times().offline_slow_hashing_1e4_per_second();
                format!("{}", crack_time)
            }
            Err(_) => String::new(),
        }
    }

    fn decode_success_page(&self) -> Element<Message> {
        column![
            text("Your decoded secret").size(30),
            vertical_space(Length::Units(20)),
            text(&self.decoded_secret),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    fn recover_backup_page(&self) -> Element<Message> {
        let idle = row![
            text("Please scan the QR codes from all required secret shares"),
            horizontal_space(Length::Fill),
            button(text("Scan code"))
                .padding(10)
                .on_press(Message::ScanCode)
        ]
        .align_items(Alignment::Center);
        let scanning = row![text(
            "Looking for QR Codes. Please position the code in front of your camera!"
        )];
        let code_list = column(
            self.scanned_codes
                .iter()
                .enumerate()
                .map(|(num, code)| {
                    container(text(format!(
                        "Secret share #{} ({} bytes)",
                        num + 1,
                        code.len()
                    )))
                    .width(Length::Fill)
                    .padding(10)
                    .style(theme::Container::Box)
                    .into()
                })
                .collect::<Vec<Element<Message>>>(),
        )
        .spacing(10);
        column![
            text("Recover a backup").size(30),
            vertical_space(Length::Units(20)),
            if self.is_scanning { scanning } else { idle },
            vertical_space(Length::Units(20)),
            scrollable(container(code_list).padding(10)),
            vertical_space(Length::Fill),
            column![
                row![
                    text("Passphrase "),
                    text("*").style(self.theme().palette().danger),
                ],
                vertical_space(Length::Units(10)),
                text_input(
                    "Enter the passphrase...",
                    &self.passphrase,
                    Message::PassphraseChanged
                )
                .padding(10)
                .width(Length::Fill),
            ],
            vertical_space(Length::Units(20)),
            row![
                button(text("Back"))
                    .padding([10, 40])
                    .on_press(Message::SwitchPage(AppPage::Welcome))
                    .style(theme::Button::Secondary),
                horizontal_space(Length::Fill),
                button(text("Decrypt"))
                    .padding([10, 40])
                    .on_press(Message::DecodeSecrets)
            ]
        ]
        .align_items(Alignment::Center)
        .into()
    }

    fn backup_results_page(&self) -> Element<Message> {
        let task_list = match &self.generated_backup {
            Some(shares) if shares.len() > 0 => column(
                shares
                    .iter()
                    .map(|share| {
                        container(
                            row![
                                text(format!("Share #{}", share.number)),
                                horizontal_space(Length::Fill),
                                button(text("Save")).on_press(Message::SaveBackup(share.number))
                            ]
                            .padding(10)
                            .align_items(Alignment::Center),
                        )
                        .style(theme::Container::Box)
                        .into()
                    })
                    .collect::<Vec<Element<Message>>>(),
            )
            .spacing(10),
            _ => column![text("Backup failed to generate.")].into(),
        };
        column![
            text("Your backup files").size(30),
            vertical_space(Length::Units(40)),
            scrollable(container(task_list).padding(20)),
            vertical_space(Length::Fill),
            row![
                horizontal_space(Length::Fill),
                button("Finish")
                    .padding([10, 40])
                    .on_press(Message::End)
                    .style(theme::Button::Secondary),
            ],
        ]
        .align_items(Alignment::Center)
        .into()
    }

    fn generating_page(&self) -> Element<Message> {
        column![
            text("Generating your backup...").size(50),
            vertical_space(Length::Units(40)),
            text("This should only take a few seconds :)")
        ]
        .align_items(Alignment::Center)
        .into()
    }

    fn create_backup_page(&self) -> Element<Message> {
        let mut next_button = button("Create").padding([10, 40]);
        if !self.passphrase.trim().is_empty() && !self.secret.trim().is_empty() {
            next_button = next_button.on_press(Message::CreateBackup)
        }

        column![
            text("Create a new backup").size(30),
            vertical_space(Length::Fill),
            row![
                text("Secret "),
                text("*").style(self.theme().palette().danger),
                horizontal_space(Length::Fill),
                text(format!("{} bytes used", self.secret.len()))
                    .style(theme::Text::Color(Color::from_rgb(0.5, 0.5, 0.5)))
            ],
            text_input(
                "Type the secret that will be backed up",
                &self.secret,
                Message::SecretChanged
            )
            .padding(10),
            vertical_space(Length::Units(20)),
            row![
                text("Passphrase "),
                text("*").style(self.theme().palette().danger),
                horizontal_space(Length::Fill),
                text(self.get_passphrase_crack_time())
                    .style(theme::Text::Color(Color::from_rgb(0.5, 0.5, 0.5)))
            ],
            row![
                text_input(
                    "Type a secure passphrase...",
                    &self.passphrase,
                    Message::PassphraseChanged
                )
                .padding(10),
                horizontal_space(Length::Units(10)),
                button(text("Generate"))
                    .padding(10)
                    .on_press(Message::GenerateSecret)
            ],
            vertical_space(Length::Units(20)),
            row![
                column![
                    row![
                        text("Backup mode "),
                        text("*").style(self.theme().palette().danger),
                    ],
                    vertical_space(Length::Units(10)),
                    pick_list(
                        &BackupType::ALL[..],
                        Some(self.backup_type),
                        Message::BackupTypeChanged
                    )
                    .padding(10)
                ]
                .width(Length::Fill),
                horizontal_space(Length::Units(10)),
                column![
                    text("Label"),
                    vertical_space(Length::Units(10)),
                    text_input(
                        "Optionally type a label...",
                        &self.label,
                        Message::LabelChanged
                    )
                    .padding(10)
                ]
                .width(Length::Fill),
            ],
            vertical_space(Length::Fill),
            row![
                button("Back")
                    .padding([10, 40])
                    .on_press(Message::SwitchPage(AppPage::Welcome))
                    .style(theme::Button::Secondary),
                horizontal_space(Length::Fill),
                next_button
            ]
            .width(Length::Fill)
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    }

    fn welcome_page(&self) -> Element<Message> {
        let create_btn = button(text("Create backup").horizontal_alignment(Horizontal::Center))
            .on_press(Message::SwitchPage(AppPage::CreateBackup))
            .padding([10, 20])
            .width(Length::Units(350));

        let restore_btn = button(text("Restore backup").horizontal_alignment(Horizontal::Center))
            .on_press(Message::SwitchPage(AppPage::RestoreBackup))
            .padding([10, 20])
            .width(Length::Units(350));

        column![
            text("Welcome to Hyperbacked").size(50),
            text("Select one of the following actions to continue"),
            vertical_space(Length::Units(48)),
            create_btn,
            restore_btn,
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    }
}

impl BackupType {
    const ALL: [BackupType; 4] = [
        BackupType::Standard,
        BackupType::Distributed { min: 2, max: 3 },
        BackupType::Distributed { min: 3, max: 5 },
        BackupType::Distributed { min: 4, max: 7 },
    ];
}

impl Display for BackupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupType::Standard => write!(f, "Standard"),
            BackupType::Distributed { min, max } => {
                write!(f, "Distributed ({} of {} shares required)", min, max)
            }
        }
    }
}
