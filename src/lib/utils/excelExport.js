import * as XLSX from 'xlsx';
import { save } from '@tauri-apps/api/dialog';
import { writeBinaryFile } from '@tauri-apps/api/fs';
import { formatDateIT, formatTime, calculateAge, calculateDurationMinutes } from './dateUtils.js';
import { calculateBMI } from './statistics.js';

/**
 * Esporta procedures in formato Excel
 */
export async function exportToExcel(procedures, filename = 'registro_tavi.xlsx') {
  try {
    // Prepara i dati per l'export
    const data = procedures.map((proc) => {
      const age = calculateAge(proc.data_nascita);
      const bmi = calculateBMI(proc.peso, proc.altezza);
      const duration = calculateDurationMinutes(proc.ora_inizio, proc.ora_fine);

      return {
        'ID': proc.id || '',
        'Nome': proc.nome || '',
        'Cognome': proc.cognome || '',
        'Data Nascita': formatDateIT(proc.data_nascita) || '',
        'Età': age !== null ? age : '',
        'Altezza (cm)': proc.altezza !== null ? proc.altezza : '',
        'Peso (kg)': proc.peso !== null ? proc.peso : '',
        'BMI': bmi !== null ? bmi.toFixed(1).replace('.', ',') : '',

        // Dati Pre-procedurali
        'FE (%)': proc.fe !== null ? proc.fe.toString().replace('.', ',') : '',
        'Vmax (m/s)': proc.vmax !== null ? proc.vmax.toString().replace('.', ',') : '',
        'Gmax (mmHg)': proc.gmax !== null ? proc.gmax.toString().replace('.', ',') : '',
        'Gmed (mmHg)': proc.gmed !== null ? proc.gmed.toString().replace('.', ',') : '',
        'AVA (cm²)': proc.ava !== null ? proc.ava.toString().replace('.', ',') : '',
        'Anulus Aortico (mm)': proc.anulus_aortico !== null ? proc.anulus_aortico.toString().replace('.', ',') : '',
        'Valvola Protesica': proc.valvola_protesica ? 'Sì' : 'No',
        'Protesica Modello': proc.protesica_modello || '',
        'Protesica Dimensione': proc.protesica_dimensione || '',

        // Dati Procedurali
        'Data Procedura': formatDateIT(proc.data_procedura) || '',
        'Ora Inizio': formatTime(proc.ora_inizio) || '',
        'Ora Fine': formatTime(proc.ora_fine) || '',
        'Durata (min)': duration !== null ? duration : '',
        'Tipo Valvola': proc.tipo_valvola || '',
        'Modello Valvola': proc.modello_valvola || '',
        'Dimensione Valvola (mm)': proc.dimensione_valvola !== null ? proc.dimensione_valvola.toString().replace('.', ',') : '',
        'Pre-dilatazione': proc.pre_dilatazione ? 'Sì' : 'No',
        'Post-dilatazione': proc.post_dilatazione ? 'Sì' : 'No',
      };
    });

    // Crea worksheet
    const worksheet = XLSX.utils.json_to_sheet(data);

    // Stile header (grassetto, background blu, testo bianco)
    const range = XLSX.utils.decode_range(worksheet['!ref']);
    for (let col = range.s.c; col <= range.e.c; col++) {
      const cellAddress = XLSX.utils.encode_cell({ r: 0, c: col });
      if (!worksheet[cellAddress]) continue;

      worksheet[cellAddress].s = {
        font: { bold: true, color: { rgb: 'FFFFFF' } },
        fill: { fgColor: { rgb: '2196F3' } },
        alignment: { horizontal: 'center', vertical: 'center' },
      };
    }

    // Imposta larghezza colonne automatica
    const columnWidths = Object.keys(data[0] || {}).map((key) => ({
      wch: Math.max(key.length, 12),
    }));
    worksheet['!cols'] = columnWidths;

    // Crea workbook
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, 'Procedure TAVI');

    // Genera buffer
    const excelBuffer = XLSX.write(workbook, { bookType: 'xlsx', type: 'array' });

    // Apri dialog per salvare
    const filePath = await save({
      defaultPath: filename,
      filters: [
        {
          name: 'Excel',
          extensions: ['xlsx'],
        },
      ],
    });

    if (filePath) {
      // Salva file
      await writeBinaryFile(filePath, new Uint8Array(excelBuffer));
      return { success: true, path: filePath };
    }

    return { success: false, cancelled: true };
  } catch (error) {
    console.error('Error exporting to Excel:', error);
    return { success: false, error: error.message };
  }
}

/**
 * Esporta statistiche in formato Excel
 */
export async function exportStatisticsToExcel(statistics, filename = 'statistiche_tavi.xlsx') {
  try {
    // Sheet 1: Statistiche Generali
    const generalData = [
      { 'Metrica': 'Totale Procedure', 'Valore': statistics.total_procedures },
      { 'Metrica': 'Durata Media (minuti)', 'Valore': statistics.average_duration_minutes.toFixed(1).replace('.', ',') },
      { 'Metrica': 'Pre-dilatazione (%)', 'Valore': statistics.pre_dilatazione_percentage.toFixed(1).replace('.', ',') },
      { 'Metrica': 'Post-dilatazione (%)', 'Valore': statistics.post_dilatazione_percentage.toFixed(1).replace('.', ',') },
      { 'Metrica': 'Balloon Expandable', 'Valore': statistics.balloon_expandable_count },
      { 'Metrica': 'Self Expandable', 'Valore': statistics.self_expandable_count },
    ];

    const ws1 = XLSX.utils.json_to_sheet(generalData);

    // Sheet 2: Parametri Emodinamici Medi
    const hemoData = [
      { 'Parametro': 'FE (%)', 'Media': statistics.average_fe ? statistics.average_fe.toFixed(1).replace('.', ',') : '-' },
      { 'Parametro': 'Vmax (m/s)', 'Media': statistics.average_vmax ? statistics.average_vmax.toFixed(2).replace('.', ',') : '-' },
      { 'Parametro': 'Gmax (mmHg)', 'Media': statistics.average_gmax ? statistics.average_gmax.toFixed(1).replace('.', ',') : '-' },
      { 'Parametro': 'Gmed (mmHg)', 'Media': statistics.average_gmed ? statistics.average_gmed.toFixed(1).replace('.', ',') : '-' },
      { 'Parametro': 'AVA (cm²)', 'Media': statistics.average_ava ? statistics.average_ava.toFixed(2).replace('.', ',') : '-' },
    ];

    const ws2 = XLSX.utils.json_to_sheet(hemoData);

    // Sheet 3: Top Modelli Valvole
    const modelData = statistics.top_valve_models.map(([model, count]) => ({
      'Modello': model,
      'Numero Procedure': count,
    }));

    const ws3 = XLSX.utils.json_to_sheet(modelData);

    // Crea workbook
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, ws1, 'Statistiche Generali');
    XLSX.utils.book_append_sheet(workbook, ws2, 'Parametri Emodinamici');
    XLSX.utils.book_append_sheet(workbook, ws3, 'Top Modelli Valvole');

    // Genera buffer
    const excelBuffer = XLSX.write(workbook, { bookType: 'xlsx', type: 'array' });

    // Apri dialog per salvare
    const filePath = await save({
      defaultPath: filename,
      filters: [
        {
          name: 'Excel',
          extensions: ['xlsx'],
        },
      ],
    });

    if (filePath) {
      // Salva file
      await writeBinaryFile(filePath, new Uint8Array(excelBuffer));
      return { success: true, path: filePath };
    }

    return { success: false, cancelled: true };
  } catch (error) {
    console.error('Error exporting statistics to Excel:', error);
    return { success: false, error: error.message };
  }
}
