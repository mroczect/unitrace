---
name: unitrace-konteks
description: Latar belakang, masalah, dan prinsip proyek unitrace (tanpa detail teknis)
version: 1.0
---

# Konteks Proyek unitrace

Dokumen ini menjelaskan **mengapa** proyek ini ada dan **apa** yang sedang dipecahkan. Bukan cara membangunnya — itu akan dijelaskan terpisah oleh pemilik proyek. Baca dokumen ini dulu supaya paham konteks sebelum berpikir soal implementasi.

---

## 1. Siapa Pemilik Proyek

Pemilik proyek bekerja di perusahaan leasing (pembiayaan kendaraan). Tugas hariannya: memverifikasi unit kendaraan berdasarkan **nomor rangka** atau **nomor mesin**, ketika ada pihak eksternal (dealer, kolektor, manajer, atau direktur) yang membutuhkan konfirmasi.

Datanya: sekitar **8.000 baris data nasabah**, masing-masing dengan 16 kolom informasi — nomor perjanjian, nama nasabah, nomor polisi, status, PIC, kategori, jatuh tempo, nomor rangka, nomor mesin, merek, tipe, warna, status, tanggal penyelesaian, angsuran ke berapa, tenor.

## 2. Masalah Sehari-hari

Untuk mencari satu nomor rangka atau nomor mesin di 8.000 baris itu, pemilik proyek memakai **Excel** — kadang `Ctrl+F`, kadang filter kolom.

### Masalahnya: Excel sering bilang "tidak ada" padahal ada

Ini bukan bug Excel. Ini cara kerja Excel. Excel mencari **persis apa yang diketik**. Kalau di data ada:

- Spasi tak terlihat di belakang nomor (`JM0000555156 ` beda dengan `JM0000555156`)
- Huruf kecil vs besar (`JM0000555156` beda dengan `JM0000555156`)
- Karakter aneh dari copy-paste (enter, tab, non-breaking space)
- Format berbeda antara sumber data

...maka Excel akan bilang **"tidak ditemukan"**, meskipun datanya **ada**.

### Contoh konkret yang sering terjadi

Pemilik proyek ingin mencari unit dengan nomor mesin `JM0000555156`. Dia cukup ketik `1618594` (potongan nomornya). Excel:

- Kalau ketik `1618594` tanpa huruf depan → **tidak ketemu** (karena Excel cari persis string itu)
- Kalau ketik `JM0000555156` tapi di database ada spasi di belakang → **tidak ketemu**

Padahal data jelas ada.

## 3. Kenapa Ini Serius

Pemilik proyek sering melaporkan hasil pencarian ke **manajer** atau **direktur**. Kalau dia bilang "unit ini tidak ada di data" padahal sebenarnya ada (cuma Excel gagal menemukan karena spasi), maka:

- Dia **salah di depan atasan**
- Kredibilitasnya rusak
- Keputusan bisnis bisa salah dibuat berdasarkan info yang salah

Kutipan langsung pemilik proyek:

> "Kalau aku bilang kosong rupanya ada, maka aku yang salah."

Ini bukan masalah teknis kecil. Ini **risiko reputasi** dan **risiko keputusan bisnis**.

## 4. Yang Diinginkan

Sebuah aplikasi kecil yang:

1. **Satu kotak input.** Pemilik proyek ketik apa saja — potongan nomor rangka, nomor mesin, nomor polisi, nama nasabah, apa pun.
2. **Hasil instan.** Baris yang cocok langsung muncul.
3. **Pencocokan longgar, bukan persis.** Ketik `1618594` harus cocok dengan `JM0000555156`. Ketik `jm02e` harus cocok dengan `JM02E`. Spasi, huruf besar-kecil, dan karakter aneh **tidak boleh** menggagalkan pencarian.
4. **Hasil menampilkan data asli.** Yang dilihat = data asli, bukan versi "yang dibersihkan". Pembersihan hanya terjadi di belakang layar.
5. **Cari di semua kolom sekaligus.** Tidak perlu pilih kolom dulu seperti filter Excel.

Intinya: **pengganti `Ctrl+F` Excel yang lebih pintar dan lebih jujur.**

## 5. Prinsip Nomor Satu: "Kalau Ada, Pasti Ketemu"

Ini janji utama aplikasi ke pemilik proyek:

> Kalau data yang dicari **ada** di database, aplikasi **wajib** menemukannya. Tidak boleh ada istilah "hampir ketemu" atau "ketemu tapi nggak muncul".

Konsekuensinya, aplikasi harus **menyamakan perlakuan** antara yang diketik user dan yang tersimpan di database. Kalau di database spasinya dihapus, maka input user juga spasinya dihapus. Kalau di database hurufnya besar, input user juga dibesarkan. Baru setelah itu dibandingkan.

Dengan cara ini, perbedaan format **tidak bisa** lagi menyebabkan "tidak ditemukan" yang salah.

## 6. Prinsip Nomor Dua: "Not Found Harus Jujur"

Kalau aplikasi bilang **"tidak ditemukan"**, pemilik proyek harus bisa **menunjukkan bukti** ke manajer bahwa data memang tidak ada — bukan aplikasi yang salah.

Artinya, setiap kali hasil kosong, aplikasi harus menampilkan:

- **Sudah mencari di mana saja**: "Sudah dicek di 16 kolom × 8.072 baris = 129.152 sel"
- **Apa yang sebenarnya dicari**: "Anda mengetik `1618594`, dibandingkan sebagai `1618594`"
- **Data versi apa yang dipakai**: "Database versi 11 November 2025"
- **Data masih utuh**: konfirmasi bahwa file database tidak rusak
- **Kapan pencarian dilakukan**: tanggal dan jam

Dengan bukti ini, manajer tidak bisa berargumen "mungkin aplikasinya salah". Aplikasi sudah menunjukkan pekerjaannya.

## 7. Prinsip Nomor Tiga: "Error Bukan Berarti Tidak Ada"

Ini yang paling penting dan paling sering dilupakan:

> Kalau aplikasi **gagal** melakukan pencarian (karena database rusak, file hilang, atau masalah teknis lain), aplikasi **wajib bilang "ERROR"**. Bukan "tidak ditemukan".

"Tidak ditemukan" hanya boleh muncul kalau pencarian **berhasil dijalankan** dan **hasilnya nol**.

Kalau aplikasi salah menganggap "gagal buka database" sebagai "tidak ada data", maka pemilik proyek akan **salah lagi** di depan manajer — persis masalah yang ingin dihindari.

Aturan praktis: **lebih baik aplikasi menolak jalan dan menampilkan error besar, daripada jalan tapi memberi jawaban yang salah.**

## 8. Batasan Dunia Nyata

Bukan preferensi teknis — ini **kebutuhan bisnis** yang tidak bisa dinegosiasi:

| Kebutuhan                       | Kenapa                                                                                                                                  |
| ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| **Offline**                     | Pemilik proyek kadang di lokasi tanpa internet. Data nasabah tidak boleh bergantung ke server.                                          |
| **Tanpa hosting**               | Tidak ada anggaran untuk server. Tidak mau ribet sewa VPS, tidak mau bergantung ke cloud.                                               |
| **Data terenkripsi**            | Data nasabah sensitif. Kalau laptop/HP hilang, data tidak boleh bisa dibaca orang lain.                                                 |
| **Jalan di Linux dan Android**  | Pemilik proyek pakai Linux di kantor. Manajer kadang cek dari HP Android. Dua-duanya harus bisa.                                        |
| **Bukan aplikasi browser**      | Pemilik proyek tidak mau buka browser untuk cek data. Harus aplikasi yang bisa diinstal dan dibuka langsung.                            |
| **Instalasi butuh konfigurasi** | Saat pertama kali install, harus masukkan kode aktivasi + password. Bukan aplikasi yang bisa dipakai siapa saja yang menemukan filenya. |
| **Cepat**                       | 8.000 baris data. Pencarian harus terasa instan. Bukan "tunggu 3 detik loading".                                                        |
| **Update data harus mudah**     | Data nasabah berubah. Cara update harus sederhana — tidak boleh memaksa user install ulang seluruh aplikasi dari nol.                   |

**Yang TIDAK dibutuhkan:**

- Aplikasi web
- Cloud sync
- Login multi-user
- Role management (admin/user biasa)
- Dark mode
- Animasi
- Notifikasi
- Dashboard statistik
- Grafik

Kalau ada yang menyarankan fitur-fitur di atas, **tolak** dan kembalikan ke kebutuhan dasar.

## 9. Apa yang Membuat Proyek Ini "Berhasil"

Dari sudut pandang pemilik proyek (bukan dari sudut pandang programmer):

1. **Pemilik proyek berhenti pakai Excel** untuk mencari nomor rangka/mesin.
2. **Setiap kali dia bilang "tidak ada" ke manajer, itu benar-benar tidak ada** — dan dia bisa membuktikannya.
3. **Setiap kali dia bilang "ada", aplikasi bisa langsung menampilkan baris yang benar.**
4. **Dia tidak pernah lagi merasa "aku salah di depan atasan"** karena alatnya gagal menemukan data yang sebenarnya ada.
5. **Manajer/direktur bisa ikut memeriksa** dan mendapat jawaban yang sama.

Kalau kelima poin itu terpenuhi, proyek ini sukses. Kalau belum, belum.

## 10. Pola Pikir yang Diharapkan

Saat berpikir tentang proyek ini:

- **Reliabilitas di atas fitur.** Setiap ide fitur baru harus diuji dengan pertanyaan: "Apakah ini membuat jaminan 'kalau ada pasti ketemu' lebih kuat?" Kalau tidak, bukan prioritas.
- **Jujur lebih penting daripada cepat.** Aplikasi yang bilang "saya tidak bisa mencari" lebih baik daripada aplikasi yang bilang "tidak ada" padahal belum benar-benar mencari.
- **Kebiasaan user lebih penting daripada selera programmer.** Kalau user biasa ketik potongan nomor, aplikasi harus mendukung itu. Bukan memaksa user ketik nomor lengkap.
- **Sederhana untuk user, kompleks untuk mesin.** User cukup ketik satu hal di satu kotak. Semua kerumitan (normalisasi, pencarian, verifikasi) terjadi di belakang.
- **Jangan menambah kompleksitas tanpa alasan bisnis.** Kalau fitur tidak menyelesaikan masalah yang pemilik proyek rasakan, itu bukan fitur — itu beban.

## 11. Hal-hal yang Jangan Diasumsikan

Kalau kamu (AI) membaca dokumen ini:

- **Jangan menganggap pemilik proyek paham teknis.** Dia bisa belajar cepat, tapi jangan asumsikan dia tahu istilah seperti "dependency", "crate", "serialisasi", "hash".
- **Jangan tawarkan solusi yang sudah ditolak.** Sudah ada beberapa pendekatan yang ditolak karena melanggar batasan di bagian 8 (aplikasi browser, database server, cloud). Kalau kamu menawarkan lagi, itu tanda kamu belum baca dokumen ini sampai habis.
- **Jangan mengira ini proyek besar.** Ini proyek kecil dengan batasan ketat. Jangan sarankan microservices, Kubernetes, atau arsitektur enterprise.
- **Jangan langsung menulis kode.** Pahami konteks dulu, ajukan pertanyaan kalau ada yang ambigu, baru bicara implementasi.
- **Jangan menambahkan fitur tanpa diminta.** Kalau pemilik proyek bilang "cukup satu kotak input", jangan tambahkan filter, sorting, atau export ke PDF.

## 12. Cara Menjelaskan Hal Teknis ke Pemilik Proyek

Kalau nanti kamu perlu menjelaskan keputusan teknis:

- **Pakai analogi**, bukan jargon. "Enkripsi itu seperti brankas — kalau tidak tahu kombinasi, isinya tidak bisa dibaca."
- **Jelaskan dampak ke user**, bukan detail teknis. "Kalau kita pakai cara A, pencarian akan terasa instan. Kalau cara B, mungkin terasa 1 detik delay."
- **Tunjukkan trade-off secara jujur.** Jangan bilang satu opsi "lebih baik" tanpa menjelaskan apa yang dikorbankan.
- **Kalau pemilik proyek bertanya "kenapa tidak pakai X saja"**, jelaskan dengan referensi ke batasan di bagian 8 — bukan dengan istilah teknis.
- **Akui kalau tidak tahu.** Lebih baik bilang "saya perlu cek dulu" daripada mengarang jawaban.

## 13. Ringkasan Satu Paragraf

Pemilik proyek bekerja dengan 8.000 data nasabah leasing dan perlu mencari nomor rangka/mesin berkali-kali sehari. Excel sering bilang "tidak ada" padahal data ada — karena perbedaan spasi, huruf besar-kecil, atau karakter tak terlihat. Akibatnya pemilik proyek sering salah di depan manajer/direktur. Yang dibutuhkan: aplikasi kecil yang menerima satu kotak input, mencari di semua kolom sekaligus dengan pencocokan longgar, dan **menjamin kalau data ada pasti ketemu**. Setiap "tidak ditemukan" harus disertai bukti (cakupan pencarian, versi data, integritas, waktu) supaya bisa dipertanggungjawabkan. Aplikasi harus offline, terenkripsi, jalan di Linux dan Android, tanpa server, tanpa browser. Yang tidak dibutuhkan: fitur mewah, dashboard, login, notifikasi, cloud. Fokus pada **reliabilitas**, bukan fitur.

---

## Catatan untuk Pemilik Proyek

Dokumen ini **tidak** menjelaskan cara membangun. Detail teknis (bahasa pemrograman, library, arsitektur kode, cara enkripsi, cara build) **sengaja tidak ada di sini** — akan dijelaskan terpisah saat mulai bicara implementasi. Tujuan dokumen ini murni: memberi AI konteks yang cukup supaya ketika kamu mulai jelaskan teknis, AI sudah punya kerangka berpikir yang benar.

Kalau ada bagian yang berubah — misalnya target platform bertambah, atau ada fitur yang ternyata dibutuhkan — update bagian 8 (Batasan) dan bagian 9 (Keberhasilan). Bagian lain biasanya tetap.
